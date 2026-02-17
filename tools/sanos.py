#!/usr/bin/env python3
"""High-level SANOS runner (Python orchestrator)."""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path


def run_cmd(cmd: list[str], cwd: Path) -> None:
    print(">", " ".join(cmd), flush=True)
    proc = subprocess.run(cmd, cwd=str(cwd))
    if proc.returncode != 0:
        raise SystemExit(proc.returncode)


def stem_name(name: str) -> str:
    return name[:-5] if name.endswith(".json") else name


def resolve_named_json(root: Path, subdir: str, name: str) -> Path:
    base = stem_name(name)
    path = root / "data" / subdir / f"{base}.json"
    if not path.exists():
        raise SystemExit(f"Missing file: {path}")
    return path


def snapshot_name_from_path(path: Path) -> str:
    name = path.name
    suffix = ".snapshot.json"
    if name.endswith(suffix):
        return name[: -len(suffix)]
    return stem_name(name)


def resolve_snapshot_catalog(root: Path, catalog: str) -> tuple[Path, list[Path]]:
    catalog_name = stem_name(catalog)
    catalog_dir = root / "data" / "snapshots" / "catalogs" / catalog_name
    if not catalog_dir.exists() or not catalog_dir.is_dir():
        raise SystemExit(f"Missing snapshot catalog directory: {catalog_dir}")

    snapshots = sorted(path for path in catalog_dir.glob("*.json") if path.is_file())
    if not snapshots:
        raise SystemExit(f"No .json snapshots found in catalog directory: {catalog_dir}")

    run_ids = set()
    for snapshot_path in snapshots:
        run_name = snapshot_name_from_path(snapshot_path)
        if run_name in run_ids:
            raise SystemExit(
                f"Duplicate snapshot run name '{run_name}' in catalog directory: {catalog_dir}"
            )
        run_ids.add(run_name)

    return catalog_dir, snapshots


def ensure_dir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def _is_non_empty_str(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _dedup_keep_order(items: list[str]) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for item in items:
        if item in seen:
            continue
        seen.add(item)
        out.append(item)
    return out


def resolve_config_catalog(root: Path, catalog: str) -> tuple[Path, dict]:
    catalog_name = stem_name(catalog)
    path = root / "data" / "config_catalogs" / f"{catalog_name}.json"
    if not path.exists():
        raise SystemExit(f"Missing config catalog file: {path}")

    payload = load_json(path)
    if not isinstance(payload, dict):
        raise SystemExit(f"Invalid config catalog format (expected object): {path}")

    default_config = payload.get("default_config")
    if not _is_non_empty_str(default_config):
        raise SystemExit(f"Invalid or missing 'default_config' in {path}")
    default_config = stem_name(default_config.strip())

    fallback_configs_raw = payload.get("fallback_configs", [])
    if fallback_configs_raw is None:
        fallback_configs_raw = []
    if not isinstance(fallback_configs_raw, list):
        raise SystemExit(f"Invalid 'fallback_configs' (expected array) in {path}")
    fallback_configs = []
    for cfg in fallback_configs_raw:
        if not _is_non_empty_str(cfg):
            raise SystemExit(f"Invalid config name in 'fallback_configs' in {path}")
        fallback_configs.append(stem_name(cfg.strip()))

    rules_raw = payload.get("rules", [])
    if rules_raw is None:
        rules_raw = []
    if not isinstance(rules_raw, list):
        raise SystemExit(f"Invalid 'rules' (expected array) in {path}")

    rules: list[dict] = []
    for idx, rule in enumerate(rules_raw):
        if not isinstance(rule, dict):
            raise SystemExit(f"Invalid rule #{idx} in {path}: expected object")
        pattern = rule.get("match")
        configs_raw = rule.get("configs")
        if not _is_non_empty_str(pattern):
            raise SystemExit(f"Invalid or missing 'match' in rule #{idx} of {path}")
        if not isinstance(configs_raw, list) or not configs_raw:
            raise SystemExit(f"Invalid or missing 'configs' in rule #{idx} of {path}")
        try:
            re.compile(pattern)
        except re.error as exc:
            raise SystemExit(
                f"Invalid regex in rule #{idx} of {path}: {pattern} ({exc})"
            ) from exc

        cfgs: list[str] = []
        for cfg in configs_raw:
            if not _is_non_empty_str(cfg):
                raise SystemExit(
                    f"Invalid config name in rule #{idx} of {path}: {cfg!r}"
                )
            cfgs.append(stem_name(cfg.strip()))

        rules.append({"match": pattern, "configs": cfgs})

    spec = {
        "default_config": default_config,
        "fallback_configs": fallback_configs,
        "rules": rules,
    }
    return path, spec


def config_candidates_for_snapshot(catalog_spec: dict, snapshot_name: str) -> list[str]:
    candidates: list[str] = []
    for rule in catalog_spec["rules"]:
        if re.search(rule["match"], snapshot_name):
            candidates.extend(rule["configs"])
            break
    candidates.append(catalog_spec["default_config"])
    candidates.extend(catalog_spec["fallback_configs"])
    return _dedup_keep_order(candidates)


def _non_negative_float(value: object, *, default: float = 0.0) -> float:
    try:
        out = float(value)
    except (TypeError, ValueError):
        return default
    if not math.isfinite(out):
        return default
    return max(0.0, out)


def _unit_interval(value: object, *, default: float = 0.0) -> float:
    out = _non_negative_float(value, default=default)
    return min(1.0, max(0.0, out))


def score_candidate_from_diagnostics(
    diagnostics_path: Path,
    *,
    w_inside: float,
    w_mae_spread_norm: float,
    w_iv_max_second_diff: float,
    w_iv_total_variation: float,
    w_bid_ask_violation: float,
    w_arb_count: float,
    w_arb_magnitude: float,
) -> tuple[float, dict]:
    diagnostics = load_json(diagnostics_path)
    summary = diagnostics.get("summary", {})
    no_arb = diagnostics.get("no_arbitrage", {})

    inside = _unit_interval(summary.get("inside_bid_ask_ratio"), default=0.0)
    mae_spread_norm = _non_negative_float(summary.get("mae_spread_norm"), default=1e9)
    iv_max_second_diff = _non_negative_float(summary.get("iv_max_second_diff"), default=0.0)
    iv_total_variation = _non_negative_float(summary.get("iv_total_variation"), default=0.0)
    max_bid_ask_violation = _non_negative_float(
        summary.get("max_bid_ask_violation"), default=0.0
    )

    mono_viol = int(_non_negative_float(no_arb.get("monotonicity_violations"), default=0.0))
    conv_viol = int(_non_negative_float(no_arb.get("convexity_violations"), default=0.0))
    cal_viol = int(_non_negative_float(no_arb.get("calendar_violations"), default=0.0))
    arb_count = mono_viol + conv_viol + cal_viol

    arb_magnitude = max(
        _non_negative_float(no_arb.get("monotonicity_max_violation"), default=0.0),
        _non_negative_float(no_arb.get("convexity_max_violation"), default=0.0),
        _non_negative_float(no_arb.get("calendar_max_violation"), default=0.0),
    )

    penalties = {
        "inside_gap": 1.0 - inside,
        "mae_spread_norm_log": math.log1p(mae_spread_norm),
        "iv_max_second_diff_log": math.log1p(iv_max_second_diff),
        "iv_total_variation_log": math.log1p(iv_total_variation),
        "bid_ask_violation_log": math.log1p(max_bid_ask_violation),
        "arb_count": float(arb_count),
        "arb_magnitude_log": math.log1p(arb_magnitude),
    }

    score = (
        w_inside * penalties["inside_gap"]
        + w_mae_spread_norm * penalties["mae_spread_norm_log"]
        + w_iv_max_second_diff * penalties["iv_max_second_diff_log"]
        + w_iv_total_variation * penalties["iv_total_variation_log"]
        + w_bid_ask_violation * penalties["bid_ask_violation_log"]
        + w_arb_count * penalties["arb_count"]
        + w_arb_magnitude * penalties["arb_magnitude_log"]
    )

    components = {
        "score": score,
        "inside_bid_ask_ratio": inside,
        "mae_spread_norm": mae_spread_norm,
        "iv_max_second_diff": iv_max_second_diff,
        "iv_total_variation": iv_total_variation,
        "max_bid_ask_violation": max_bid_ask_violation,
        "arb_count": arb_count,
        "arb_magnitude": arb_magnitude,
        "penalties": penalties,
    }
    return score, components


def _quality_comment(summary: dict) -> str:
    inside = float(summary.get("inside_bid_ask_ratio", 0.0))
    mae_norm = float(summary.get("mae_spread_norm", 0.0))
    mae_mid = float(summary.get("mae_mid", 0.0))
    rmse_mid = float(summary.get("rmse_mid", 0.0))
    mae_iv = summary.get("mae_iv")
    rmse_iv = summary.get("rmse_iv")

    lines: list[str] = []
    if inside >= 0.99:
        lines.append("La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread).")
    elif inside >= 0.95:
        lines.append("La calibration est globalement coherente avec le bid/ask mais quelques quotes sortent du spread.")
    else:
        lines.append("Le taux inside bid/ask est faible: la calibration merite un reglage des hyperparametres.")

    if mae_norm <= 0.5:
        lines.append("Les residus normalises sont globalement contenus par rapport au demi-spread.")
    elif mae_norm <= 1.0:
        lines.append("Les residus normalises restent raisonnables mais sont proches de la largeur des spreads.")
    else:
        lines.append("Les residus normalises sont eleves face aux spreads: la robustesse locale est a ameliorer.")

    lines.append(
        f"Erreur en prix: MAE={mae_mid:.3e}, RMSE={rmse_mid:.3e}."
    )
    if mae_iv is not None and rmse_iv is not None:
        lines.append(f"Erreur en volatilite implicite: MAE={float(mae_iv):.3e}, RMSE={float(rmse_iv):.3e}.")
    if summary.get("iv_total_variation") is not None and summary.get("iv_max_second_diff") is not None:
        lines.append(
            f"Rugosite IV (global): TV={float(summary['iv_total_variation']):.3e}, "
            f"max seconde diff={float(summary['iv_max_second_diff']):.3e}."
        )
    return " ".join(lines)


def _fmt_opt(value: float | None, fmt: str) -> str:
    if value is None:
        return "-"
    return format(float(value), fmt)


def _top_smoothness_improvements(cmp_data: dict, n: int = 3) -> list[dict]:
    rows = cmp_data.get("per_maturity", []) if isinstance(cmp_data, dict) else []
    scored = []
    for row in rows:
        delta = row.get("delta_max_second_diff")
        if delta is None:
            continue
        scored.append((float(delta), row))
    scored.sort(key=lambda x: x[0])
    return [row for _, row in scored[: max(0, n)]]


def build_markdown_report(
    reports_dir: Path,
    run_id: str,
    config_path: Path,
    snapshot_path: Path,
    surfaces_dir: Path,
    images_dir: Path,
    diagnostics_path: Path,
    surface_path: Path,
    plots_generated: bool,
) -> Path:
    ensure_dir(reports_dir)
    report_path = reports_dir / f"{run_id}.md"

    diagnostics = load_json(diagnostics_path)
    summary = diagnostics.get("summary", {})
    no_arb = diagnostics.get("no_arbitrage", {})
    smooth_cmp = diagnostics.get("smoothness_comparison")
    per = diagnostics.get("per_maturity", [])
    image_rel = Path(os.path.relpath(images_dir, reports_dir))
    surface_rel = Path(os.path.relpath(surfaces_dir, reports_dir))
    image_rel_txt = image_rel.as_posix()
    surface_rel_txt = surface_rel.as_posix()

    lines: list[str] = []
    lines.append(f"# Calibration Report - `{run_id}`")
    lines.append("")
    lines.append("## Inputs")
    lines.append(f"- Config: `{config_path}`")
    lines.append(f"- Snapshot: `{snapshot_path}`")
    lines.append("")
    lines.append("## Outputs")
    lines.append(f"- Surface JSON: `{surface_path}`")
    lines.append(f"- Diagnostics JSON: `{diagnostics_path}`")
    lines.append(f"- Surface folder: `{surfaces_dir}`")
    lines.append(f"- Images folder: `{images_dir}`")
    lines.append("")
    lines.append("## Global Metrics")
    lines.append(f"- Number of quotes: `{summary.get('n_quotes', 0)}`")
    lines.append(f"- Objective value: `{float(summary.get('objective_value', 0.0)):.6e}`")
    lines.append(f"- Inside bid/ask ratio: `{float(summary.get('inside_bid_ask_ratio', 0.0)):.2%}`")
    lines.append(
        f"- Bid/ask violation (max / mean): "
        f"`{float(summary.get('max_bid_ask_violation', 0.0)):.3e}` / "
        f"`{float(summary.get('mean_bid_ask_violation', 0.0)):.3e}`"
    )
    lines.append(f"- MAE(mid): `{float(summary.get('mae_mid', 0.0)):.3e}`")
    lines.append(f"- RMSE(mid): `{float(summary.get('rmse_mid', 0.0)):.3e}`")
    lines.append(f"- MAE(residual/half-spread): `{float(summary.get('mae_spread_norm', 0.0)):.3f}`")
    if summary.get("iv_total_variation") is not None:
        lines.append(f"- IV total variation: `{float(summary.get('iv_total_variation')):.3e}`")
    if summary.get("iv_max_second_diff") is not None:
        lines.append(f"- IV max second diff: `{float(summary.get('iv_max_second_diff')):.3e}`")
    if summary.get("mae_iv") is not None and summary.get("rmse_iv") is not None:
        lines.append(f"- MAE(iv): `{float(summary.get('mae_iv')):.3e}`")
        lines.append(f"- RMSE(iv): `{float(summary.get('rmse_iv')):.3e}`")
    lines.append("")
    lines.append("## No-Arbitrage Diagnostics")
    lines.append(
        f"- Strike monotonicity violation (max / mean / count): "
        f"`{float(no_arb.get('monotonicity_max_violation', 0.0)):.3e}` / "
        f"`{float(no_arb.get('monotonicity_mean_violation', 0.0)):.3e}` / "
        f"`{int(no_arb.get('monotonicity_violations', 0))}`"
    )
    lines.append(
        f"- Strike convexity violation (max / mean / count): "
        f"`{float(no_arb.get('convexity_max_violation', 0.0)):.3e}` / "
        f"`{float(no_arb.get('convexity_mean_violation', 0.0)):.3e}` / "
        f"`{int(no_arb.get('convexity_violations', 0))}`"
    )
    lines.append(
        f"- Calendar violation (max / mean / count): "
        f"`{float(no_arb.get('calendar_max_violation', 0.0)):.3e}` / "
        f"`{float(no_arb.get('calendar_mean_violation', 0.0)):.3e}` / "
        f"`{int(no_arb.get('calendar_violations', 0))}`"
    )
    lines.append("")
    lines.append("### Interpretation automatique")
    lines.append(_quality_comment(summary))
    lines.append("")
    if isinstance(smooth_cmp, dict):
        lines.append("## Oscillation Analysis (Before/After)")
        lines.append(
            f"- Baseline IV smoothness (no init): TV=`{float(smooth_cmp.get('baseline_total_variation', 0.0)):.3e}`, "
            f"max second diff=`{float(smooth_cmp.get('baseline_max_second_diff', 0.0)):.3e}`"
        )
        lines.append(
            f"- Current IV smoothness (linear-density init): TV=`{float(smooth_cmp.get('current_total_variation', 0.0)):.3e}`, "
            f"max second diff=`{float(smooth_cmp.get('current_max_second_diff', 0.0)):.3e}`"
        )
        lines.append(
            f"- Delta (current - baseline): TV=`{float(smooth_cmp.get('delta_total_variation', 0.0)):.3e}`, "
            f"max second diff=`{float(smooth_cmp.get('delta_max_second_diff', 0.0)):.3e}`"
        )
        if float(smooth_cmp.get("delta_max_second_diff", 0.0)) < 0.0:
            lines.append(
                "Conclusion: the IV oscillation level decreases after enabling linear-density initialization."
            )
        else:
            lines.append(
                "Conclusion: oscillation did not improve globally; inspect per-maturity rows below."
            )
        top_rows = _top_smoothness_improvements(smooth_cmp, n=4)
        if top_rows:
            lines.append("")
            lines.append("Most improved maturities (delta max second diff):")
            for row in top_rows:
                lines.append(
                    "- "
                    f"T={float(row.get('maturity', 0.0)):.6f}, "
                    f"delta={float(row.get('delta_max_second_diff', 0.0)):.3e}, "
                    f"baseline strike={_fmt_opt(row.get('baseline_strike_max_second_diff'), '.4f')}, "
                    f"current strike={_fmt_opt(row.get('current_strike_max_second_diff'), '.4f')}"
                )
        lines.append("")

    lines.append("## Per Maturity")
    lines.append(
        "| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | "
        "TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |"
    )
    lines.append(
        "|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    )
    for row in per:
        proj_needed = row.get("linear_projection_needed")
        proj_needed_txt = (
            "yes" if proj_needed is True else ("no" if proj_needed is False else "-")
        )
        lines.append(
            "| "
            f"{float(row.get('maturity', 0.0)):.6f} | "
            f"{int(row.get('n_quotes', 0))} | "
            f"{float(row.get('inside_bid_ask_ratio', 0.0)):.2%} | "
            f"{float(row.get('max_bid_ask_violation', 0.0)):.3e} | "
            f"{float(row.get('mae_mid', 0.0)):.3e} | "
            f"{float(row.get('mae_spread_norm', 0.0)):.3f} | "
            f"{_fmt_opt(row.get('iv_total_variation'), '.3e')} | "
            f"{_fmt_opt(row.get('iv_max_second_diff'), '.3e')} | "
            f"{float(row.get('density_mass', 0.0)):.6f} | "
            f"{float(row.get('density_mean', 0.0)):.6f} | "
            f"{proj_needed_txt} | "
            f"{_fmt_opt(row.get('linear_projection_l1'), '.3e')} |"
        )
    lines.append("")
    lines.append("Linear density discretization used:")
    lines.append(
        "- Node-based second-difference on strike grid: "
        "`p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, "
        "with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`."
    )
    lines.append(
        "- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under "
        "`p*>=0`, `sum(p*)=1`, `sum(p* K)=1`."
    )
    lines.append("")
    lines.append("## Figures And Interpretation")
    if plots_generated:
        lines.append(f"- Price smiles: `{image_rel_txt}/{run_id}_smiles_fit.png`")
        lines.append("  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.")
        lines.append(f"- IV smiles: `{image_rel_txt}/{run_id}_smiles_iv_fit.png`")
        lines.append("  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.")
        lines.append(f"- Residual heatmap: `{image_rel_txt}/{run_id}_residual_heatmap.png`")
        lines.append("  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.")
        lines.append(f"- Quality summary: `{image_rel_txt}/{run_id}_quality_summary.png`")
        lines.append("  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.")
        lines.append(f"- Surface heatmap: `{image_rel_txt}/{run_id}_surface_heatmap.png`")
        lines.append("  Interpretation: continuite globale de la surface en maturite/strike.")
        lines.append(f"- Density comparison: `{image_rel_txt}/{run_id}_density_comparison.png`")
        lines.append("  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).")
    else:
        lines.append("- Plots skipped (`--no-plot`).")
    lines.append("")
    lines.append("## Reconstructability")
    lines.append(
        f"`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` "
        "sans la config ni le snapshot d'origine."
    )
    lines.append(f"- Surface JSON folder: `{surface_rel_txt}`")
    lines.append("")

    report_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return report_path


def cleanup_unused_run_artifacts(
    *,
    surfaces_root: Path,
    images_root: Path,
    reports_dir: Path,
    attempted_run_ids: list[str],
    keep_run_ids: set[str],
) -> None:
    removed = 0
    failed: list[str] = []
    for run_id in _dedup_keep_order(attempted_run_ids):
        if run_id in keep_run_ids:
            continue

        surfaces_dir = surfaces_root / run_id
        images_dir = images_root / run_id
        report_path = reports_dir / f"{run_id}.md"

        for path in [surfaces_dir, images_dir]:
            if path.exists():
                try:
                    shutil.rmtree(path, ignore_errors=False)
                    removed += 1
                except OSError as exc:
                    failed.append(f"{path} ({exc})")
        if report_path.exists():
            try:
                report_path.unlink()
                removed += 1
            except OSError as exc:
                failed.append(f"{report_path} ({exc})")

    if removed > 0:
        print(f"Cleanup: removed {removed} unused artifacts.")
    if failed:
        print("Cleanup warning: failed to remove some artifacts:")
        for item in failed:
            print(f"- {item}")


def run_single_snapshot(
    *,
    root: Path,
    config_path: Path,
    snapshot_path: Path,
    cfg_name: str,
    snap_name: str,
    surfaces_root: Path,
    images_root: Path,
    reports_dir: Path,
    args: argparse.Namespace,
) -> tuple[str, Path]:
    run_id = f"{snap_name}__{cfg_name}"
    surfaces_dir = surfaces_root / run_id
    images_dir = images_root / run_id
    ensure_dir(surfaces_dir)
    ensure_dir(images_dir)

    # 1) Optional sanity check of the snapshot.
    if not args.no_validate:
        run_cmd(
            [
                "cargo",
                "run",
                "-p",
                "sanos-cli",
                "--",
                "validate-snapshot",
                "--snapshot",
                str(snapshot_path),
            ],
            cwd=root,
        )

    # 2) Calibration via Rust CLI + exports:
    #    q.json, report.json, surface.json, diagnostics.json
    run_cmd(
        [
            "cargo",
            "run",
            "-p",
            "sanos-cli",
            "--",
            "calibrate",
            "--snapshot",
            str(snapshot_path),
            "--config",
            str(config_path),
            "--out",
            str(surfaces_dir),
            "--n-maturities",
            str(args.n_maturities),
            "--n-strikes",
            str(args.n_strikes),
        ],
        cwd=root,
    )

    diagnostics_path = surfaces_dir / "diagnostics.json"
    surface_path = surfaces_dir / "surface.json"

    if not diagnostics_path.exists():
        raise SystemExit(f"Expected diagnostics file not found: {diagnostics_path}")
    if not surface_path.exists():
        raise SystemExit(f"Expected surface file not found: {surface_path}")

    # 3) Ensure `surface.json` is actually re-hydratable into a SanosSurface.
    run_cmd(
        [
            "cargo",
            "run",
            "-p",
            "sanos-cli",
            "--",
            "validate-surface",
            "--surface",
            str(surface_path),
        ],
        cwd=root,
    )

    # 4) Optional plots
    plots_generated = False
    if not args.no_plot:
        plot_script = root / "tools" / "plot_calibration_quality.py"
        if not plot_script.exists():
            raise SystemExit(f"Plot script not found: {plot_script}")

        run_cmd(
            [
                args.python,
                str(plot_script),
                "--diagnostics",
                str(diagnostics_path),
                "--surface",
                str(surface_path),
                "--outdir",
                str(images_dir),
                "--prefix",
                run_id,
            ],
            cwd=root,
        )
        plots_generated = True

    report_path = build_markdown_report(
        reports_dir=reports_dir,
        run_id=run_id,
        config_path=config_path,
        snapshot_path=snapshot_path,
        surfaces_dir=surfaces_dir,
        images_dir=images_dir,
        diagnostics_path=diagnostics_path,
        surface_path=surface_path,
        plots_generated=plots_generated,
    )

    print("\nDONE")
    print("Config     :", config_path)
    print("Snapshot   :", snapshot_path)
    print("Surface dir:", surfaces_dir)
    print("Image dir  :", images_dir)
    print("Surface    :", surface_path)
    print("Diagnostics:", diagnostics_path)
    print("Report     :", report_path)
    return run_id, report_path


def run_snapshot_with_config_candidates(
    *,
    root: Path,
    config_candidates: list[tuple[str, Path]],
    snapshot_path: Path,
    snap_name: str,
    surfaces_root: Path,
    images_root: Path,
    reports_dir: Path,
    args: argparse.Namespace,
    selection_policy: str,
) -> tuple[str, Path, str, float]:
    if not config_candidates:
        raise SystemExit(f"No config candidates provided for snapshot: {snap_name}")
    if selection_policy not in {"first-success", "best-score"}:
        raise SystemExit(f"Invalid selection policy: {selection_policy}")

    attempt_errors: list[str] = []
    attempted_run_ids: list[str] = []
    total = len(config_candidates)
    successes: list[dict] = []

    for idx, (cfg_name, config_path) in enumerate(config_candidates, start=1):
        run_id_candidate = f"{snap_name}__{cfg_name}"
        attempted_run_ids.append(run_id_candidate)
        if total > 1:
            print(f"Trying config [{idx}/{total}] for {snap_name}: {cfg_name}")
        try:
            run_id, report_path = run_single_snapshot(
                root=root,
                config_path=config_path,
                snapshot_path=snapshot_path,
                cfg_name=cfg_name,
                snap_name=snap_name,
                surfaces_root=surfaces_root,
                images_root=images_root,
                reports_dir=reports_dir,
                args=args,
            )
            diagnostics_path = surfaces_root / run_id / "diagnostics.json"
            score, components = score_candidate_from_diagnostics(
                diagnostics_path,
                w_inside=args.score_w_inside,
                w_mae_spread_norm=args.score_w_mae_spread_norm,
                w_iv_max_second_diff=args.score_w_iv_max_second_diff,
                w_iv_total_variation=args.score_w_iv_total_variation,
                w_bid_ask_violation=args.score_w_bid_ask_violation,
                w_arb_count=args.score_w_arb_count,
                w_arb_magnitude=args.score_w_arb_magnitude,
            )
            print(
                "Score "
                f"{cfg_name}: {score:.6f} "
                f"(inside={components['inside_bid_ask_ratio']:.4f}, "
                f"mae_norm={components['mae_spread_norm']:.4g}, "
                f"d2={components['iv_max_second_diff']:.4g}, "
                f"arb={components['arb_count']})"
            )

            successes.append(
                {
                    "run_id": run_id,
                    "report_path": report_path,
                    "cfg_name": cfg_name,
                    "score": score,
                }
            )

            if selection_policy == "first-success":
                if not args.keep_attempt_artifacts:
                    cleanup_unused_run_artifacts(
                        surfaces_root=surfaces_root,
                        images_root=images_root,
                        reports_dir=reports_dir,
                        attempted_run_ids=attempted_run_ids,
                        keep_run_ids={run_id},
                    )
                if idx > 1:
                    print(f"Selected config '{cfg_name}' for snapshot '{snap_name}'.")
                return run_id, report_path, cfg_name, score
        except SystemExit as exc:
            reason = str(exc.code) if exc.code is not None else "SystemExit"
            attempt_errors.append(f"{cfg_name}: {reason}")
            print(f"Config '{cfg_name}' failed for '{snap_name}': {reason}")
        except Exception as exc:  # noqa: BLE001
            reason = str(exc)
            attempt_errors.append(f"{cfg_name}: {reason}")
            print(f"Config '{cfg_name}' failed for '{snap_name}': {reason}")

    if successes:
        best = min(successes, key=lambda x: x["score"])
        if not args.keep_attempt_artifacts:
            cleanup_unused_run_artifacts(
                surfaces_root=surfaces_root,
                images_root=images_root,
                reports_dir=reports_dir,
                attempted_run_ids=attempted_run_ids,
                keep_run_ids={best["run_id"]},
            )
        print(
            f"Selected best-score config '{best['cfg_name']}' for '{snap_name}' "
            f"(score={best['score']:.6f})."
        )
        return (
            best["run_id"],
            best["report_path"],
            best["cfg_name"],
            best["score"],
        )

    if not args.keep_attempt_artifacts:
        cleanup_unused_run_artifacts(
            surfaces_root=surfaces_root,
            images_root=images_root,
            reports_dir=reports_dir,
            attempted_run_ids=attempted_run_ids,
            keep_run_ids=set(),
        )

    joined = "; ".join(attempt_errors) if attempt_errors else "unknown error"
    raise SystemExit(f"all config attempts failed ({joined})")


def write_catalog_summary(
    *,
    reports_dir: Path,
    catalog_name: str,
    cfg_label: str,
    config_selection: str,
    selection_policy: str,
    snapshot_paths: list[Path],
    run_results: dict[str, dict[str, object]],
    failures: dict[str, str],
) -> Path:
    ensure_dir(reports_dir)
    summary_path = reports_dir / f"{catalog_name}__{cfg_label}__summary.md"

    lines: list[str] = []
    lines.append(f"# Catalog Run Summary - `{catalog_name}__{cfg_label}`")
    lines.append("")
    lines.append("## Inputs")
    lines.append(f"- Config selection: `{config_selection}`")
    lines.append(f"- Selection policy: `{selection_policy}`")
    lines.append(f"- Catalog: `{catalog_name}`")
    lines.append(f"- Snapshots count: `{len(snapshot_paths)}`")
    lines.append("")
    lines.append("## Results")
    lines.append(f"- Success: `{len(run_results)}`")
    lines.append(f"- Failed: `{len(failures)}`")
    lines.append("")
    lines.append("| Snapshot | Status | Config | Score | Report | Error |")
    lines.append("|---|---|---|---:|---|---|")

    for snapshot_path in snapshot_paths:
        snap_name = snapshot_name_from_path(snapshot_path)
        run_data = run_results.get(snap_name)
        if run_data is not None:
            report_path = run_data["report_path"]
            cfg_used = run_data["config"]
            score = run_data.get("score")
            score_txt = "-" if score is None else f"{float(score):.6f}"
            rel_report = Path(os.path.relpath(report_path, reports_dir))
            lines.append(
                f"| `{snap_name}` | success | `{cfg_used}` | `{score_txt}` | `{rel_report}` | - |"
            )
            continue
        error = failures.get(snap_name, "Unknown error").replace("|", "\\|")
        lines.append(f"| `{snap_name}` | failed | - | - | - | `{error}` |")

    lines.append("")
    summary_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return summary_path


def run_pipeline(args: argparse.Namespace) -> None:
    root = Path(args.root).resolve()
    selection_policy = args.selection_policy
    for field in [
        "score_w_inside",
        "score_w_mae_spread_norm",
        "score_w_iv_max_second_diff",
        "score_w_iv_total_variation",
        "score_w_bid_ask_violation",
        "score_w_arb_count",
        "score_w_arb_magnitude",
    ]:
        value = getattr(args, field)
        if not isinstance(value, float) or not math.isfinite(value) or value < 0.0:
            raise SystemExit(
                f"Invalid score weight '{field}': {value} (must be finite and >= 0)"
            )
    config_catalog_path: Path | None = None
    config_catalog_spec: dict | None = None
    if args.config is not None:
        cfg_name = stem_name(args.config)
        config_path = resolve_named_json(root, "configs", cfg_name)
        cfg_label = cfg_name
        config_selection = str(config_path)
    else:
        catalog_name = stem_name(args.config_catalog)
        config_catalog_path, config_catalog_spec = resolve_config_catalog(root, catalog_name)
        cfg_name = ""
        config_path = Path()
        cfg_label = f"catalog_{catalog_name}"
        config_selection = str(config_catalog_path)

    if args.snapshot is not None:
        snapshot_path = resolve_named_json(root, "snapshots", stem_name(args.snapshot))
        snap_name = snapshot_name_from_path(snapshot_path)
        if args.config is not None:
            run_single_snapshot(
                root=root,
                config_path=config_path,
                snapshot_path=snapshot_path,
                cfg_name=cfg_name,
                snap_name=snap_name,
                surfaces_root=root / "data" / "surfaces",
                images_root=root / "data" / "images",
                reports_dir=root / "data" / "reports",
                args=args,
            )
            return

        assert config_catalog_spec is not None
        config_names = config_candidates_for_snapshot(config_catalog_spec, snap_name)
        config_candidates = [
            (name, resolve_named_json(root, "configs", name))
            for name in config_names
        ]
        print(f"Snapshot '{snap_name}' config candidates: {', '.join(config_names)}")
        run_snapshot_with_config_candidates(
            root=root,
            config_candidates=config_candidates,
            snapshot_path=snapshot_path,
            snap_name=snap_name,
            surfaces_root=root / "data" / "surfaces",
            images_root=root / "data" / "images",
            reports_dir=root / "data" / "reports",
            args=args,
            selection_policy=selection_policy,
        )
        return

    snapshot_catalog_name = stem_name(args.snapshot_catalog)
    catalog_dir, snapshot_paths = resolve_snapshot_catalog(root, snapshot_catalog_name)
    surfaces_root = root / "data" / "surfaces" / "catalogs" / snapshot_catalog_name
    images_root = root / "data" / "images" / "catalogs" / snapshot_catalog_name
    reports_dir = root / "data" / "reports" / "catalogs" / snapshot_catalog_name
    ensure_dir(surfaces_root)
    ensure_dir(images_root)
    ensure_dir(reports_dir)

    run_results: dict[str, dict[str, object]] = {}
    failures: dict[str, str] = {}

    total = len(snapshot_paths)
    print(f"Running snapshot catalog '{snapshot_catalog_name}' ({total} snapshots)")
    print(f"Catalog dir: {catalog_dir}")
    if config_catalog_path is not None:
        print(f"Config catalog: {config_catalog_path}")
    else:
        print(f"Config: {config_path}")
    print(f"Selection policy: {selection_policy}")

    for i, snapshot_path in enumerate(snapshot_paths, start=1):
        snap_name = snapshot_name_from_path(snapshot_path)
        print(f"\n[{i}/{total}] Snapshot: {snapshot_path}")
        try:
            if args.config is not None:
                run_id, report_path = run_single_snapshot(
                    root=root,
                    config_path=config_path,
                    snapshot_path=snapshot_path,
                    cfg_name=cfg_name,
                    snap_name=snap_name,
                    surfaces_root=surfaces_root,
                    images_root=images_root,
                    reports_dir=reports_dir,
                    args=args,
                )
                score, _ = score_candidate_from_diagnostics(
                    surfaces_root / run_id / "diagnostics.json",
                    w_inside=args.score_w_inside,
                    w_mae_spread_norm=args.score_w_mae_spread_norm,
                    w_iv_max_second_diff=args.score_w_iv_max_second_diff,
                    w_iv_total_variation=args.score_w_iv_total_variation,
                    w_bid_ask_violation=args.score_w_bid_ask_violation,
                    w_arb_count=args.score_w_arb_count,
                    w_arb_magnitude=args.score_w_arb_magnitude,
                )
                run_results[snap_name] = {
                    "report_path": report_path,
                    "config": cfg_name,
                    "score": score,
                }
            else:
                assert config_catalog_spec is not None
                config_names = config_candidates_for_snapshot(config_catalog_spec, snap_name)
                config_candidates = [
                    (name, resolve_named_json(root, "configs", name))
                    for name in config_names
                ]
                print(f"Config candidates: {', '.join(config_names)}")
                _, report_path, cfg_used, score = run_snapshot_with_config_candidates(
                    root=root,
                    config_candidates=config_candidates,
                    snapshot_path=snapshot_path,
                    snap_name=snap_name,
                    surfaces_root=surfaces_root,
                    images_root=images_root,
                    reports_dir=reports_dir,
                    args=args,
                    selection_policy=selection_policy,
                )
                run_results[snap_name] = {
                    "report_path": report_path,
                    "config": cfg_used,
                    "score": score,
                }
        except SystemExit as exc:
            reason = str(exc.code) if exc.code is not None else "SystemExit"
            failures[snap_name] = reason
            print(f"FAILED {snap_name}: {reason}")
        except Exception as exc:  # noqa: BLE001
            failures[snap_name] = str(exc)
            print(f"FAILED {snap_name}: {exc}")

    summary_path = write_catalog_summary(
        reports_dir=reports_dir,
        catalog_name=snapshot_catalog_name,
        cfg_label=cfg_label,
        config_selection=config_selection,
        selection_policy=selection_policy,
        snapshot_paths=snapshot_paths,
        run_results=run_results,
        failures=failures,
    )

    print("\nCATALOG DONE")
    print("Catalog   :", snapshot_catalog_name)
    if config_catalog_path is not None:
        print("Config    :", config_catalog_path)
    else:
        print("Config    :", config_path)
    print("Reports   :", reports_dir)
    print("Summary   :", summary_path)
    print("Success   :", len(run_results))
    print("Failed    :", len(failures))

    if failures:
        raise SystemExit(1)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="SANOS Python orchestrator (Rust calibration + Python plots)"
    )
    parser.add_argument(
        "--root",
        default=".",
        help="Repository root (default: current directory)",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    run = sub.add_parser("run", help="Run calibration by config/snapshot names")
    run_cfg = run.add_mutually_exclusive_group(required=True)
    run_cfg.add_argument(
        "--config",
        help="Config file name in data/configs (without .json)",
    )
    run_cfg.add_argument(
        "--config-catalog",
        help="Config catalog file name in data/config_catalogs (without .json)",
    )
    run_input = run.add_mutually_exclusive_group(required=True)
    run_input.add_argument("--snapshot", help="Snapshot file name in data/snapshots (without .json)")
    run_input.add_argument(
        "--snapshot-catalog",
        help="Snapshot catalog directory name in data/snapshots/catalogs",
    )
    run.add_argument("--n-maturities", type=int, default=41, help="Export grid maturities")
    run.add_argument("--n-strikes", type=int, default=81, help="Export grid strikes")
    run.add_argument("--python", default=sys.executable, help="Python executable for plot script")
    run.add_argument("--no-validate", action="store_true", help="Skip snapshot validation step")
    run.add_argument("--no-plot", action="store_true", help="Skip plot generation")
    run.add_argument(
        "--keep-attempt-artifacts",
        action="store_true",
        help="Keep artifacts from non-selected config attempts (debug mode)",
    )
    run.add_argument(
        "--selection-policy",
        choices=["first-success", "best-score"],
        default="first-success",
        help="Config selection strategy when multiple config candidates are available",
    )
    run.add_argument(
        "--score-w-inside",
        type=float,
        default=100.0,
        help="Score weight for (1 - inside bid/ask ratio)",
    )
    run.add_argument(
        "--score-w-mae-spread-norm",
        type=float,
        default=8.0,
        help="Score weight for log(1 + MAE residual normalized by half-spread)",
    )
    run.add_argument(
        "--score-w-iv-max-second-diff",
        type=float,
        default=1.0,
        help="Score weight for log(1 + max second diff of model IV)",
    )
    run.add_argument(
        "--score-w-iv-total-variation",
        type=float,
        default=0.3,
        help="Score weight for log(1 + total variation of model IV)",
    )
    run.add_argument(
        "--score-w-bid-ask-violation",
        type=float,
        default=20.0,
        help="Score weight for log(1 + max bid/ask violation)",
    )
    run.add_argument(
        "--score-w-arb-count",
        type=float,
        default=1_000_000.0,
        help="Score weight for arbitrage violation count",
    )
    run.add_argument(
        "--score-w-arb-magnitude",
        type=float,
        default=1_000_000.0,
        help="Score weight for log(1 + max arbitrage violation magnitude)",
    )
    run.set_defaults(func=run_pipeline)

    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
