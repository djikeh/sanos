#!/usr/bin/env python3
"""High-level SANOS runner (Python orchestrator)."""

from __future__ import annotations

import argparse
import json
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
    return " ".join(lines)


def build_markdown_report(
    root: Path,
    run_id: str,
    config_path: Path,
    snapshot_path: Path,
    surfaces_dir: Path,
    images_dir: Path,
    diagnostics_path: Path,
    surface_path: Path,
    plots_generated: bool,
) -> Path:
    reports_dir = root / "data" / "reports"
    ensure_dir(reports_dir)
    report_path = reports_dir / f"{run_id}.md"

    diagnostics = load_json(diagnostics_path)
    summary = diagnostics.get("summary", {})
    per = diagnostics.get("per_maturity", [])
    image_rel = Path("..") / "images" / run_id
    surface_rel = Path("..") / "surfaces" / run_id

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
    lines.append(f"- Inside bid/ask ratio: `{float(summary.get('inside_bid_ask_ratio', 0.0)):.2%}`")
    lines.append(f"- MAE(mid): `{float(summary.get('mae_mid', 0.0)):.3e}`")
    lines.append(f"- RMSE(mid): `{float(summary.get('rmse_mid', 0.0)):.3e}`")
    lines.append(f"- MAE(residual/half-spread): `{float(summary.get('mae_spread_norm', 0.0)):.3f}`")
    if summary.get("mae_iv") is not None and summary.get("rmse_iv") is not None:
        lines.append(f"- MAE(iv): `{float(summary.get('mae_iv')):.3e}`")
        lines.append(f"- RMSE(iv): `{float(summary.get('rmse_iv')):.3e}`")
    lines.append("")
    lines.append("### Interpretation automatique")
    lines.append(_quality_comment(summary))
    lines.append("")
    lines.append("## Per Maturity")
    lines.append("| T | n_quotes | inside | MAE(mid) | RMSE(mid) | MAE(norm) | MAE(iv) |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|")
    for row in per:
        mae_iv = row.get("mae_iv")
        mae_iv_txt = f"{float(mae_iv):.3e}" if mae_iv is not None else "-"
        lines.append(
            "| "
            f"{float(row.get('maturity', 0.0)):.6f} | "
            f"{int(row.get('n_quotes', 0))} | "
            f"{float(row.get('inside_bid_ask_ratio', 0.0)):.2%} | "
            f"{float(row.get('mae_mid', 0.0)):.3e} | "
            f"{float(row.get('rmse_mid', 0.0)):.3e} | "
            f"{float(row.get('mae_spread_norm', 0.0)):.3f} | "
            f"{mae_iv_txt} |"
        )
    lines.append("")
    lines.append("## Figures And Interpretation")
    if plots_generated:
        lines.append(f"- Price smiles: `{image_rel}/{run_id}_smiles_fit.png`")
        lines.append("  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.")
        lines.append(f"- IV smiles: `{image_rel}/{run_id}_smiles_iv_fit.png`")
        lines.append("  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.")
        lines.append(f"- Residual heatmap: `{image_rel}/{run_id}_residual_heatmap.png`")
        lines.append("  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.")
        lines.append(f"- Quality summary: `{image_rel}/{run_id}_quality_summary.png`")
        lines.append("  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.")
        lines.append(f"- Surface heatmap: `{image_rel}/{run_id}_surface_heatmap.png`")
        lines.append("  Interpretation: continuite globale de la surface en maturite/strike.")
        lines.append(f"- Density comparison: `{image_rel}/{run_id}_density_comparison.png`")
        lines.append("  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).")
    else:
        lines.append("- Plots skipped (`--no-plot`).")
    lines.append("")
    lines.append("## Reconstructability")
    lines.append(
        f"`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` "
        "sans la config ni le snapshot d'origine."
    )
    lines.append(f"- Surface JSON folder: `{surface_rel}`")
    lines.append("")

    report_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return report_path


def run_pipeline(args: argparse.Namespace) -> None:
    root = Path(args.root).resolve()

    cfg_name = stem_name(args.config)
    snap_name = stem_name(args.snapshot)
    run_id = f"{snap_name}__{cfg_name}"

    config_path = resolve_named_json(root, "configs", cfg_name)
    snapshot_path = resolve_named_json(root, "snapshots", snap_name)

    surfaces_dir = root / "data" / "surfaces" / run_id
    images_dir = root / "data" / "images" / run_id
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
        root=root,
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
    run.add_argument("--snapshot", required=True, help="Snapshot file name in data/snapshots (without .json)")
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
