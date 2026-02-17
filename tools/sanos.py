#!/usr/bin/env python3
"""High-level SANOS runner (Python orchestrator)."""

from __future__ import annotations

import argparse
import json
import os
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


def write_catalog_summary(
    *,
    reports_dir: Path,
    catalog_name: str,
    cfg_name: str,
    config_path: Path,
    snapshot_paths: list[Path],
    run_results: dict[str, Path],
    failures: dict[str, str],
) -> Path:
    ensure_dir(reports_dir)
    summary_path = reports_dir / f"{catalog_name}__{cfg_name}__summary.md"

    lines: list[str] = []
    lines.append(f"# Catalog Run Summary - `{catalog_name}__{cfg_name}`")
    lines.append("")
    lines.append("## Inputs")
    lines.append(f"- Config: `{config_path}`")
    lines.append(f"- Catalog: `{catalog_name}`")
    lines.append(f"- Snapshots count: `{len(snapshot_paths)}`")
    lines.append("")
    lines.append("## Results")
    lines.append(f"- Success: `{len(run_results)}`")
    lines.append(f"- Failed: `{len(failures)}`")
    lines.append("")
    lines.append("| Snapshot | Status | Report | Error |")
    lines.append("|---|---|---|---|")

    for snapshot_path in snapshot_paths:
        snap_name = snapshot_name_from_path(snapshot_path)
        report_path = run_results.get(snap_name)
        if report_path is not None:
            rel_report = Path(os.path.relpath(report_path, reports_dir))
            lines.append(f"| `{snap_name}` | success | `{rel_report}` | - |")
            continue
        error = failures.get(snap_name, "Unknown error").replace("|", "\\|")
        lines.append(f"| `{snap_name}` | failed | - | `{error}` |")

    lines.append("")
    summary_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return summary_path


def run_pipeline(args: argparse.Namespace) -> None:
    root = Path(args.root).resolve()
    cfg_name = stem_name(args.config)
    config_path = resolve_named_json(root, "configs", cfg_name)

    if args.snapshot is not None:
        snap_name = stem_name(args.snapshot)
        snapshot_path = resolve_named_json(root, "snapshots", snap_name)
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

    catalog_name = stem_name(args.snapshot_catalog)
    catalog_dir, snapshot_paths = resolve_snapshot_catalog(root, catalog_name)
    surfaces_root = root / "data" / "surfaces" / "catalogs" / catalog_name
    images_root = root / "data" / "images" / "catalogs" / catalog_name
    reports_dir = root / "data" / "reports" / "catalogs" / catalog_name
    ensure_dir(surfaces_root)
    ensure_dir(images_root)
    ensure_dir(reports_dir)

    run_results: dict[str, Path] = {}
    failures: dict[str, str] = {}

    total = len(snapshot_paths)
    print(f"Running snapshot catalog '{catalog_name}' ({total} snapshots)")
    print(f"Catalog dir: {catalog_dir}")
    for i, snapshot_path in enumerate(snapshot_paths, start=1):
        snap_name = snapshot_name_from_path(snapshot_path)
        print(f"\n[{i}/{total}] Snapshot: {snapshot_path}")
        try:
            _, report_path = run_single_snapshot(
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
            run_results[snap_name] = report_path
        except SystemExit as exc:
            reason = str(exc.code) if exc.code is not None else "SystemExit"
            failures[snap_name] = reason
            print(f"FAILED {snap_name}: {reason}")
        except Exception as exc:  # noqa: BLE001
            failures[snap_name] = str(exc)
            print(f"FAILED {snap_name}: {exc}")

    summary_path = write_catalog_summary(
        reports_dir=reports_dir,
        catalog_name=catalog_name,
        cfg_name=cfg_name,
        config_path=config_path,
        snapshot_paths=snapshot_paths,
        run_results=run_results,
        failures=failures,
    )

    print("\nCATALOG DONE")
    print("Catalog   :", catalog_name)
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
    run.add_argument("--config", required=True, help="Config file name in data/configs (without .json)")
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
    run.set_defaults(func=run_pipeline)

    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
