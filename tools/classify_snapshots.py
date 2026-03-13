#!/usr/bin/env python3
"""Classify snapshots by best calibration config.

Scans all diagnostics.json files produced by the calibration grid
(5 configs x 30 snapshots) and builds a classification table:
  config -> list of snapshots it calibrates best
  + list of uncalibrable snapshots.

Smoothness of IV and density are first-class quality criteria.
"""

from __future__ import annotations

import argparse
import json
import math
import subprocess
import sys
from pathlib import Path


CONFIGS = [
    "tight_spread",
    "zero_spread_vega",
    "strong_wings",
    "sparse_robust",
    "extreme_stress",
]
CATALOGS = ["sabr", "svi_raw", "total_variance"]


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def _non_negative_float(value: object, *, default: float = 0.0) -> float:
    try:
        out = float(value)
    except (TypeError, ValueError):
        return default
    if not math.isfinite(out):
        return default
    return max(0.0, out)


def _unit_interval(value: object, *, default: float = 0.0) -> float:
    return min(1.0, max(0.0, _non_negative_float(value, default=default)))


def load_snapshot_meta(snapshot_path: Path) -> dict:
    payload = load_json(snapshot_path)
    spreads: list[float] = []
    for maturity in payload.get("maturities", []):
        for quote in maturity.get("quotes", []):
            bid_iv = _non_negative_float(quote.get("bid_iv"))
            ask_iv = _non_negative_float(quote.get("ask_iv"))
            spreads.append(max(0.0, ask_iv - bid_iv))
    if not spreads:
        return {
            "all_zero_spread": False,
            "zero_spread_ratio": 0.0,
            "n_quotes": 0,
        }
    zero_count = sum(1 for spread in spreads if spread <= 1e-15)
    return {
        "all_zero_spread": zero_count == len(spreads),
        "zero_spread_ratio": zero_count / len(spreads),
        "n_quotes": len(spreads),
    }


def score_diagnostics(diag: dict, *, snapshot_meta: dict) -> tuple[float, dict]:
    """Score a calibration result with smoothness-heavy weights."""
    summary = diag.get("summary", {})
    no_arb = diag.get("no_arbitrage", {})
    per_mat = diag.get("per_maturity", [])
    all_zero_spread = bool(snapshot_meta.get("all_zero_spread", False))

    inside = _unit_interval(summary.get("inside_bid_ask_ratio"), default=0.0)
    mae_spread_norm = _non_negative_float(summary.get("mae_spread_norm"), default=1e9)
    mae_iv = _non_negative_float(summary.get("mae_iv"), default=0.0)
    rmse_iv = _non_negative_float(summary.get("rmse_iv"), default=0.0)
    iv_max_d2 = _non_negative_float(summary.get("iv_max_second_diff"), default=0.0)
    iv_tv = _non_negative_float(summary.get("iv_total_variation"), default=0.0)
    max_ba_viol = _non_negative_float(summary.get("max_bid_ask_violation"), default=0.0)

    mono_v = int(_non_negative_float(no_arb.get("monotonicity_violations"), default=0.0))
    conv_v = int(_non_negative_float(no_arb.get("convexity_violations"), default=0.0))
    cal_v = int(_non_negative_float(no_arb.get("calendar_violations"), default=0.0))
    arb_count = mono_v + conv_v + cal_v

    arb_mag = max(
        _non_negative_float(no_arb.get("monotonicity_max_violation"), default=0.0),
        _non_negative_float(no_arb.get("convexity_max_violation"), default=0.0),
        _non_negative_float(no_arb.get("calendar_max_violation"), default=0.0),
    )

    # Density health: check for oscillatory density
    density_penalty = 0.0
    for m in per_mat:
        near_zero = int(m.get("density_near_zero_atoms", 0))
        # Estimate total atoms from grid (density_mass ~ 1.0 if well-calibrated)
        d_min = float(m.get("density_min", 0.0))
        d_max = float(m.get("density_max", 1.0))
        if d_max > 0 and d_min >= 0:
            # Large ratio of near-zero atoms indicates sparse/oscillatory density
            # Use a heuristic: if near_zero > 10 and density_max is large, penalty
            if near_zero > 15 and d_max > 0.5:
                density_penalty = max(density_penalty, 30.0)
            elif near_zero > 10 and d_max > 0.3:
                density_penalty = max(density_penalty, 15.0)

    # Smoothness-heavy scoring
    W_INSIDE = 50.0
    W_MAE_SPREAD = 5.0
    W_MAE_IV_ZERO_SPREAD = 400.0
    W_IV_D2 = 20.0
    W_IV_TV = 10.0
    W_BA_VIOL = 200.0 if all_zero_spread else 15.0
    W_ARB_COUNT = 1_000_000.0
    W_ARB_MAG = 1_000_000.0

    if all_zero_spread:
        fit_penalty = W_MAE_IV_ZERO_SPREAD * math.log1p(mae_iv)
        fit_basis = "mae_iv"
    else:
        fit_penalty = (
            W_INSIDE * (1.0 - inside)
            + W_MAE_SPREAD * math.log1p(mae_spread_norm)
        )
        fit_basis = "inside_and_mae_spread_norm"

    score = (
        fit_penalty
        + W_IV_D2 * math.log1p(iv_max_d2)
        + W_IV_TV * math.log1p(iv_tv)
        + W_BA_VIOL * math.log1p(max_ba_viol)
        + W_ARB_COUNT * arb_count
        + W_ARB_MAG * math.log1p(arb_mag)
        + density_penalty
    )

    components = {
        "all_zero_spread": all_zero_spread,
        "zero_spread_ratio": float(snapshot_meta.get("zero_spread_ratio", 0.0)),
        "inside": inside,
        "mae_spread_norm": mae_spread_norm,
        "mae_iv": mae_iv,
        "rmse_iv": rmse_iv,
        "iv_max_d2": iv_max_d2,
        "iv_tv": iv_tv,
        "max_ba_viol": max_ba_viol,
        "arb_count": arb_count,
        "arb_mag": arb_mag,
        "density_penalty": density_penalty,
        "fit_basis": fit_basis,
        "score": score,
    }
    return score, components


def classify(components: dict) -> str:
    if components["arb_count"] > 0:
        return "echec"

    if components["all_zero_spread"]:
        if (
            components["mae_iv"] <= 0.005
            and components["max_ba_viol"] <= 1e-3
            and components["iv_max_d2"] < 3.0
            and components["iv_tv"] < 1.0
        ):
            return "excellent"
        if (
            components["mae_iv"] <= 0.015
            and components["max_ba_viol"] <= 5e-3
            and components["iv_max_d2"] < 10.0
            and components["iv_tv"] < 3.0
        ):
            return "acceptable"
        if (
            components["mae_iv"] <= 0.05
            and components["max_ba_viol"] <= 2e-2
            and components["iv_max_d2"] < 25.0
        ):
            return "mediocre"
        return "fragile"

    inside = components["inside"]
    iv_max_d2 = components["iv_max_d2"]
    iv_tv = components["iv_tv"]
    if inside >= 0.99 and iv_max_d2 < 3.0 and iv_tv < 1.0:
        return "excellent"
    if inside >= 0.95 and iv_max_d2 < 10.0 and iv_tv < 3.0:
        return "acceptable"
    if inside >= 0.80 and iv_max_d2 < 15.0:
        return "mediocre"
    return "fragile"


def build_snapshot_meta(root: Path, catalogs: list[str]) -> dict[str, dict]:
    meta_by_snapshot: dict[str, dict] = {}
    for cat in catalogs:
        snap_dir = root / "data" / "snapshots" / "catalogs" / cat
        for snapshot_path in sorted(snap_dir.glob("*.json")):
            snap_name = snapshot_path.name.replace(".snapshot.json", "").replace(".json", "")
            meta_by_snapshot[snap_name] = {
                **load_snapshot_meta(snapshot_path),
                "catalog": cat,
                "snapshot_path": snapshot_path,
            }
    return meta_by_snapshot


def _fmt_pct_or_na(value: float, *, enabled: bool) -> str:
    if not enabled:
        return "n/a"
    return f"{value:.2%}"


def _fmt_float_or_na(value: float, *, enabled: bool, fmt: str) -> str:
    if not enabled:
        return "n/a"
    return format(value, fmt)


def generate_best_plots(best_per_snapshot: dict[str, tuple[str, dict]], *, root: Path, python_exec: str) -> list[str]:
    plot_script = root / "tools" / "plot_calibration_quality.py"
    if not plot_script.exists():
        return [f"Plot script missing: {plot_script}"]

    failures: list[str] = []
    for snap_name, (_, comp) in sorted(best_per_snapshot.items()):
        diagnostics_path = Path(comp["diagnostics_path"])
        surface_path = Path(comp["surface_path"])
        images_dir = Path(comp["images_dir"])
        run_id = diagnostics_path.parent.name
        images_dir.mkdir(parents=True, exist_ok=True)
        cmd = [
            python_exec,
            str(plot_script),
            "--diagnostics",
            str(diagnostics_path),
            "--surface",
            str(surface_path),
            "--outdir",
            str(images_dir),
            "--prefix",
            run_id,
        ]
        proc = subprocess.run(cmd, cwd=str(root), check=False)
        if proc.returncode != 0:
            failures.append(f"{snap_name}: plot generation failed with code {proc.returncode}")
    return failures


def build_report(
    *,
    root: Path,
    configs: list[str],
    results: dict[str, dict[str, dict]],
    failures: dict[str, list[str]],
    best_per_snapshot: dict[str, tuple[str, dict]],
    config_to_snapshots: dict[str, list[tuple[str, dict]]],
    fragile: list[tuple[str, str, dict]],
    real_failures: list[tuple[str, str, dict]],
    plot_best: bool,
    plot_failures: list[str],
) -> str:
    report_dir = root / "data" / "reports"
    image_root_rel = Path("..") / "images" / "catalogs"
    zero_spread_count = sum(
        1 for _, (_, comp) in best_per_snapshot.items() if comp["all_zero_spread"]
    )

    lines: list[str] = []
    lines.append("# Classification des snapshots par config optimale")
    lines.append("")
    lines.append("## Methode")
    lines.append(
        "- `echec` designe uniquement un `FAIL` de calibration ou une violation d'arbitrage (`arb_count > 0`)."
    )
    lines.append(
        "- Pour les snapshots a spread strictement nul (`bid_iv == ask_iv` partout), `inside` et `mae_spread_norm` sont ignores."
    )
    lines.append(
        "- Les snapshots a spread nul sont classes avec `MAE(iv)`, `max_bid_ask_violation`, `iv_max_d2` et `iv_tv`."
    )
    lines.append(
        "- La categorie `fragile` regroupe les calibrations sans arbitrage mais dont le fit ou la regularite restent insuffisants."
    )
    lines.append("")
    lines.append("## Resume")
    lines.append(f"- Snapshots analyses: `{len(best_per_snapshot)}`")
    lines.append(f"- Snapshots a spread nul: `{zero_spread_count}`")
    lines.append(f"- Snapshots fragiles: `{len(fragile)}`")
    lines.append(f"- Echecs reels: `{len(real_failures)}`")
    if plot_best:
        lines.append("- Plots: generation demandee pour la meilleure config de chaque snapshot.")
        if plot_failures:
            lines.append(f"- Echecs de plots: `{len(plot_failures)}`")
        else:
            lines.append("- Echecs de plots: `0`")
    lines.append("")
    lines.append("## Criteres de qualite")
    lines.append("### Spreads strictement positifs")
    lines.append("- **excellent** : inside >= 99%, iv_max_d2 < 3.0, iv_tv < 1.0, 0 arb")
    lines.append("- **acceptable** : inside >= 95%, iv_max_d2 < 10.0, iv_tv < 3.0, 0 arb")
    lines.append("- **mediocre** : inside >= 80%, iv_max_d2 < 15.0, 0 arb")
    lines.append("- **fragile** : calibration sans arbitrage, mais hors seuils ci-dessus")
    lines.append("- **echec** : `FAIL` ou arbitrage detecte")
    lines.append("")
    lines.append("### Spreads nuls")
    lines.append("- **excellent** : mae_iv <= 0.005, max_bid_ask_violation <= 1e-3, iv_max_d2 < 3.0, iv_tv < 1.0, 0 arb")
    lines.append("- **acceptable** : mae_iv <= 0.015, max_bid_ask_violation <= 5e-3, iv_max_d2 < 10.0, iv_tv < 3.0, 0 arb")
    lines.append("- **mediocre** : mae_iv <= 0.05, max_bid_ask_violation <= 2e-2, iv_max_d2 < 25.0, 0 arb")
    lines.append("- **fragile** : calibration sans arbitrage, mais hors seuils ci-dessus")
    lines.append("- **echec** : `FAIL` ou arbitrage detecte")
    lines.append("")
    lines.append("## Tableau de classification")
    lines.append("")

    for cfg in configs:
        snaps = config_to_snapshots[cfg]
        lines.append(f"### Config: `{cfg}` ({len(snaps)} snapshots)")
        lines.append("")
        if not snaps:
            lines.append("_Aucun snapshot assigne._")
            lines.append("")
            continue
        lines.append(
            "| Snapshot | Regime | Quality | Inside | MAE norm | MAE iv | IV max d2 | IV TV | Max BA viol | Arb | Score |"
        )
        lines.append(
            "|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|"
        )
        for snap, comp in sorted(snaps, key=lambda x: x[1]["score"]):
            spread_is_positive = not comp["all_zero_spread"]
            lines.append(
                f"| `{snap}` "
                f"| {'zero-spread' if comp['all_zero_spread'] else 'spread'} "
                f"| {comp['quality']} "
                f"| {_fmt_pct_or_na(comp['inside'], enabled=spread_is_positive)} "
                f"| {_fmt_float_or_na(comp['mae_spread_norm'], enabled=spread_is_positive, fmt='.4f')} "
                f"| {comp['mae_iv']:.4f} "
                f"| {comp['iv_max_d2']:.4f} "
                f"| {comp['iv_tv']:.4f} "
                f"| {comp['max_ba_viol']:.4e} "
                f"| {comp['arb_count']} "
                f"| {comp['score']:.2f} |"
            )
        lines.append("")

    lines.append("## Snapshots fragiles (meilleure config sans arbitrage)")
    lines.append("")
    if not fragile:
        lines.append("_Aucun snapshot fragile._")
    else:
        lines.append(
            "| Snapshot | Best Config | Regime | Inside | MAE iv | IV max d2 | Score | Images |"
        )
        lines.append("|---|---|---|---:|---:|---:|---:|---|")
        for snap, best_cfg, comp in sorted(fragile, key=lambda x: x[2]["score"]):
            img_dir = image_root_rel / comp["catalog"] / Path(comp["images_dir"]).name
            inside_txt = _fmt_pct_or_na(comp["inside"], enabled=not comp["all_zero_spread"])
            lines.append(
                f"| `{snap}` "
                f"| `{best_cfg}` "
                f"| {'zero-spread' if comp['all_zero_spread'] else 'spread'} "
                f"| {inside_txt} "
                f"| {comp['mae_iv']:.4f} "
                f"| {comp['iv_max_d2']:.4f} "
                f"| {comp['score']:.2f} "
                f"| `{img_dir.as_posix()}` |"
            )
    lines.append("")

    lines.append("## Snapshots en echec reel")
    lines.append("")
    if not real_failures:
        lines.append("_Aucun snapshot en echec reel._")
    else:
        lines.append(
            "| Snapshot | Best Config | Regime | Arb | Score |"
        )
        lines.append("|---|---|---|---:|---:|")
        for snap, best_cfg, comp in sorted(real_failures, key=lambda x: x[2]["score"]):
            lines.append(
                f"| `{snap}` "
                f"| `{best_cfg}` "
                f"| {'zero-spread' if comp['all_zero_spread'] else 'spread'} "
                f"| {comp['arb_count']} "
                f"| {comp['score']:.2f} |"
            )
    lines.append("")

    snap_no_success = {
        snap_name for snap_name in failures if snap_name not in results or not results[snap_name]
    }
    if snap_no_success:
        lines.append("## Snapshots sans aucune calibration reussie")
        lines.append("")
        for snap_name in sorted(snap_no_success):
            lines.append(f"- `{snap_name}`: {', '.join(failures.get(snap_name, []))}")
        lines.append("")

    if plot_best and plot_failures:
        lines.append("## Plots non generes")
        lines.append("")
        for failure in plot_failures:
            lines.append(f"- {failure}")
        lines.append("")

    lines.append("## Matrice complete des scores (snapshot x config)")
    lines.append("")
    header = "| Snapshot |"
    for cfg in configs:
        header += f" `{cfg}` |"
    lines.append(header)
    lines.append("|---|" + "---:|" * len(configs))

    for snap in sorted(results.keys()):
        row = f"| `{snap}` |"
        for cfg in configs:
            if cfg in results[snap]:
                score = results[snap][cfg]["score"]
                quality = results[snap][cfg]["quality"]
                marker = " **" if quality == "excellent" else ""
                marker_end = "**" if quality == "excellent" else ""
                row += f" {marker}{score:.1f}{marker_end} |"
            else:
                row += " FAIL |"
        lines.append(row)
    lines.append("")
    return "\n".join(lines) + "\n"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Classify SANOS calibration runs and optionally generate best-run plots."
    )
    parser.add_argument(
        "--plot-best",
        action="store_true",
        help="Generate plots for the best config selected for each snapshot.",
    )
    parser.add_argument(
        "--python",
        default=sys.executable,
        help="Python executable used for plot generation.",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    root = Path(__file__).resolve().parent.parent
    surfaces_root = root / "data" / "surfaces" / "catalogs"
    snapshot_meta = build_snapshot_meta(root, CATALOGS)

    # Collect all results: {snapshot_name: {config_name: (score, components, quality)}}
    results: dict[str, dict[str, dict]] = {}
    failures: dict[str, list[str]] = {}  # snapshot -> list of failed configs

    for cat in CATALOGS:
        cat_dir = surfaces_root / cat
        if not cat_dir.exists():
            continue
        for cfg in CONFIGS:
            # Find all run dirs matching *__cfg
            for run_dir in sorted(cat_dir.iterdir()):
                if not run_dir.is_dir():
                    continue
                if not run_dir.name.endswith(f"__{cfg}"):
                    continue
                snap_name = run_dir.name[: -(len(cfg) + 2)]
                diag_path = run_dir / "diagnostics.json"

                if not diag_path.exists():
                    failures.setdefault(snap_name, []).append(cfg)
                    continue

                try:
                    diag = load_json(diag_path)
                    meta = snapshot_meta.get(
                        snap_name,
                        {
                            "all_zero_spread": False,
                            "zero_spread_ratio": 0.0,
                            "snapshot_path": None,
                            "catalog": cat,
                        },
                    )
                    score, comp = score_diagnostics(diag, snapshot_meta=meta)
                    quality = classify(comp)
                    results.setdefault(snap_name, {})[cfg] = {
                        **comp,
                        "catalog": cat,
                        "run_dir": str(run_dir),
                        "diagnostics_path": str(diag_path),
                        "surface_path": str(run_dir / "surface.json"),
                        "images_dir": str(root / "data" / "images" / "catalogs" / cat / run_dir.name),
                        "quality": quality,
                    }
                except Exception as exc:
                    failures.setdefault(snap_name, []).append(f"{cfg}({exc})")

    # For each snapshot, find best config
    best_per_snapshot: dict[str, tuple[str, dict]] = {}
    for snap, cfg_results in sorted(results.items()):
        best_cfg = min(cfg_results, key=lambda c: cfg_results[c]["score"])
        best_per_snapshot[snap] = (best_cfg, cfg_results[best_cfg])

    # Invert: config -> list of snapshots
    config_to_snapshots: dict[str, list[tuple[str, dict]]] = {c: [] for c in CONFIGS}
    fragile: list[tuple[str, str, dict]] = []
    real_failures: list[tuple[str, str, dict]] = []

    for snap, (best_cfg, comp) in sorted(best_per_snapshot.items()):
        if comp["quality"] == "echec":
            real_failures.append((snap, best_cfg, comp))
        elif comp["quality"] == "fragile":
            config_to_snapshots[best_cfg].append((snap, comp))
            fragile.append((snap, best_cfg, comp))
        else:
            config_to_snapshots[best_cfg].append((snap, comp))

    plot_failures: list[str] = []
    if args.plot_best:
        plot_failures = generate_best_plots(best_per_snapshot, root=root, python_exec=args.python)

    output = build_report(
        root=root,
        configs=CONFIGS,
        results=results,
        failures=failures,
        best_per_snapshot=best_per_snapshot,
        config_to_snapshots=config_to_snapshots,
        fragile=fragile,
        real_failures=real_failures,
        plot_best=args.plot_best,
        plot_failures=plot_failures,
    )

    out_path = root / "data" / "reports" / "classification.md"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(output, encoding="utf-8")
    print(output)
    print(f"\nClassification written to: {out_path}")


if __name__ == "__main__":
    main()
