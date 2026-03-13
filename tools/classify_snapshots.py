#!/usr/bin/env python3
"""Classify snapshots by best calibration config.

Scans all diagnostics.json files produced by the calibration grid
(5 configs x 30 snapshots) and builds a classification table:
  config -> list of snapshots it calibrates best
  + list of uncalibrable snapshots.

Smoothness of IV and density are first-class quality criteria.
"""

from __future__ import annotations

import json
import math
import re
import sys
from pathlib import Path


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def score_diagnostics(diag: dict) -> tuple[float, dict]:
    """Score a calibration result with smoothness-heavy weights."""
    summary = diag.get("summary", {})
    no_arb = diag.get("no_arbitrage", {})
    per_mat = diag.get("per_maturity", [])

    inside = min(1.0, max(0.0, float(summary.get("inside_bid_ask_ratio", 0.0))))
    mae_spread_norm = max(0.0, float(summary.get("mae_spread_norm", 1e9)))
    iv_max_d2 = max(0.0, float(summary.get("iv_max_second_diff") or 0.0))
    iv_tv = max(0.0, float(summary.get("iv_total_variation") or 0.0))
    max_ba_viol = max(0.0, float(summary.get("max_bid_ask_violation", 0.0)))

    mono_v = int(max(0, float(no_arb.get("monotonicity_violations", 0))))
    conv_v = int(max(0, float(no_arb.get("convexity_violations", 0))))
    cal_v = int(max(0, float(no_arb.get("calendar_violations", 0))))
    arb_count = mono_v + conv_v + cal_v

    arb_mag = max(
        max(0.0, float(no_arb.get("monotonicity_max_violation", 0.0))),
        max(0.0, float(no_arb.get("convexity_max_violation", 0.0))),
        max(0.0, float(no_arb.get("calendar_max_violation", 0.0))),
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
    W_IV_D2 = 20.0
    W_IV_TV = 10.0
    W_BA_VIOL = 15.0
    W_ARB_COUNT = 1_000_000.0
    W_ARB_MAG = 1_000_000.0

    score = (
        W_INSIDE * (1.0 - inside)
        + W_MAE_SPREAD * math.log1p(mae_spread_norm)
        + W_IV_D2 * math.log1p(iv_max_d2)
        + W_IV_TV * math.log1p(iv_tv)
        + W_BA_VIOL * math.log1p(max_ba_viol)
        + W_ARB_COUNT * arb_count
        + W_ARB_MAG * math.log1p(arb_mag)
        + density_penalty
    )

    components = {
        "inside": inside,
        "mae_spread_norm": mae_spread_norm,
        "iv_max_d2": iv_max_d2,
        "iv_tv": iv_tv,
        "max_ba_viol": max_ba_viol,
        "arb_count": arb_count,
        "arb_mag": arb_mag,
        "density_penalty": density_penalty,
        "score": score,
    }
    return score, components


def classify(inside: float, arb_count: int, iv_max_d2: float, iv_tv: float) -> str:
    if arb_count > 0:
        return "echec"
    if inside < 0.50 or iv_max_d2 > 30.0:
        return "echec"
    if inside >= 0.99 and iv_max_d2 < 3.0 and iv_tv < 1.0:
        return "excellent"
    if inside >= 0.95 and iv_max_d2 < 10.0 and iv_tv < 3.0:
        return "acceptable"
    if inside >= 0.80 and iv_max_d2 < 15.0:
        return "mediocre"
    return "echec"


def main() -> None:
    root = Path(__file__).resolve().parent.parent
    surfaces_root = root / "data" / "surfaces" / "catalogs"

    configs = ["tight_spread", "zero_spread_vega", "strong_wings", "sparse_robust", "extreme_stress"]
    catalogs = ["sabr", "svi_raw", "total_variance"]

    # Collect all results: {snapshot_name: {config_name: (score, components, quality)}}
    results: dict[str, dict[str, dict]] = {}
    failures: dict[str, list[str]] = {}  # snapshot -> list of failed configs

    for cat in catalogs:
        cat_dir = surfaces_root / cat
        if not cat_dir.exists():
            continue
        for cfg in configs:
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
                    score, comp = score_diagnostics(diag)
                    quality = classify(
                        comp["inside"], comp["arb_count"], comp["iv_max_d2"], comp["iv_tv"]
                    )
                    results.setdefault(snap_name, {})[cfg] = {
                        **comp,
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
    config_to_snapshots: dict[str, list[tuple[str, dict]]] = {c: [] for c in configs}
    uncalibrable: list[tuple[str, str, dict]] = []

    for snap, (best_cfg, comp) in sorted(best_per_snapshot.items()):
        if comp["quality"] == "echec":
            uncalibrable.append((snap, best_cfg, comp))
        else:
            config_to_snapshots[best_cfg].append((snap, comp))

    # Print classification table
    lines: list[str] = []
    lines.append("# Classification des snapshots par config optimale")
    lines.append("")
    lines.append("## Criteres de qualite")
    lines.append("- **excellent** : inside >= 99%, 0 arb, iv_max_d2 < 3.0, iv_tv < 1.0")
    lines.append("- **acceptable** : inside >= 95%, 0 arb, iv_max_d2 < 10.0, iv_tv < 3.0")
    lines.append("- **mediocre** : inside >= 80%, 0 arb, iv_max_d2 < 15.0")
    lines.append("- **echec** : arb > 0 OU inside < 50% OU iv_max_d2 > 30.0")
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
            "| Snapshot | Quality | Inside | IV max d2 | IV TV | MAE norm | Arb | Density pen. | Score |"
        )
        lines.append(
            "|---|---|---:|---:|---:|---:|---:|---:|---:|"
        )
        for snap, comp in sorted(snaps, key=lambda x: x[1]["score"]):
            lines.append(
                f"| `{snap}` "
                f"| {comp['quality']} "
                f"| {comp['inside']:.2%} "
                f"| {comp['iv_max_d2']:.4f} "
                f"| {comp['iv_tv']:.4f} "
                f"| {comp['mae_spread_norm']:.4f} "
                f"| {comp['arb_count']} "
                f"| {comp['density_penalty']:.0f} "
                f"| {comp['score']:.2f} |"
            )
        lines.append("")

    lines.append("## Snapshots incalibrables (echec avec toutes les configs)")
    lines.append("")
    if not uncalibrable:
        lines.append("_Aucun snapshot en echec total._")
    else:
        lines.append(
            "| Snapshot | Best Config | Quality | Inside | IV max d2 | Arb | Score |"
        )
        lines.append("|---|---|---|---:|---:|---:|---:|")
        for snap, best_cfg, comp in sorted(uncalibrable, key=lambda x: x[2]["score"]):
            lines.append(
                f"| `{snap}` "
                f"| `{best_cfg}` "
                f"| {comp['quality']} "
                f"| {comp['inside']:.2%} "
                f"| {comp['iv_max_d2']:.4f} "
                f"| {comp['arb_count']} "
                f"| {comp['score']:.2f} |"
            )
    lines.append("")

    # Failed calibrations summary
    all_failed_snaps = set()
    for snap, cfg_results in results.items():
        pass
    snap_no_success = set()
    for snap in failures:
        if snap not in results or not results[snap]:
            snap_no_success.add(snap)

    if snap_no_success:
        lines.append("## Snapshots sans aucune calibration reussie")
        lines.append("")
        for snap in sorted(snap_no_success):
            lines.append(f"- `{snap}`: {', '.join(failures.get(snap, []))}")
        lines.append("")

    # Full score matrix
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
                s = results[snap][cfg]["score"]
                q = results[snap][cfg]["quality"]
                marker = " **" if q == "excellent" else ""
                marker_end = "**" if q == "excellent" else ""
                row += f" {marker}{s:.1f}{marker_end} |"
            else:
                row += " FAIL |"
        lines.append(row)
    lines.append("")

    output = "\n".join(lines) + "\n"

    out_path = root / "data" / "reports" / "classification.md"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(output, encoding="utf-8")
    print(output)
    print(f"\nClassification written to: {out_path}")


if __name__ == "__main__":
    main()
