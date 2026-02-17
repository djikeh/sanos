# sanos

Minimal workspace to calibrate SANOS surfaces from IV snapshots.

## Rust CLI

Build and run with `cargo run -p sanos-cli -- <command> ...`.

### Commands

1. Create a default calibration config:
```bash
cargo run -p sanos-cli -- init-config --out data/configs/default.json
```
This template is runnable out of the box (`solver = "Microlp"`, `objective = "HardBidAsk"`).

2. Validate a snapshot:
```bash
cargo run -p sanos-cli -- validate-snapshot --snapshot data/snapshots/my_snapshot.json
```

3. Run calibration + JSON exports:
```bash
cargo run -p sanos-cli -- calibrate \
  --snapshot data/snapshots/my_snapshot.json \
  --config data/configs/default.json \
  --out data/surfaces/my_run \
  --n-maturities 41 \
  --n-strikes 81
```

This writes:
- `report.json`
- `q.json`
- `surface.json` (dense and pretty-printed, includes `reconstruction` to rebuild `SanosSurface`)
- `diagnostics.json`

4. Validate and reconstruct a previously exported surface:
```bash
cargo run -p sanos-cli -- validate-surface --surface data/surfaces/my_run/surface.json
```

## Python orchestrator (`sanos run`)

The top-level orchestrator is Python and does not create Rust dependencies on `tools/`.

```bash
python tools/sanos.py run --config default --snapshot my_snapshot
```

Catalog run:
```bash
python tools/sanos.py run --config default --snapshot-catalog total_variance
```

Shortcuts:
- Windows (from repo root): `sanos.cmd run --config default --snapshot my_snapshot`
- Linux/macOS (from repo root): `./sanos run --config default --snapshot my_snapshot`

Single snapshot resolves:
- `data/configs/default.json`
- `data/snapshots/my_snapshot.json`

Then it:
1. Validates the snapshot.
2. Runs calibration via Rust CLI.
3. Writes JSON outputs in `data/surfaces/<snapshot>__<config>/`.
4. Generates calibration quality plots in `data/images/<snapshot>__<config>/`.
5. Generates a Markdown report in `data/reports/<snapshot>__<config>.md`.

Catalog run resolves:
- `data/configs/default.json`
- `data/snapshots/catalogs/total_variance/*.json`

Then it runs each snapshot and writes:
- Surfaces: `data/surfaces/catalogs/<catalog>/<snapshot>__<config>/`
- Images: `data/images/catalogs/<catalog>/<snapshot>__<config>/`
- Per-run reports: `data/reports/catalogs/<catalog>/<snapshot>__<config>.md`
- Batch summary report: `data/reports/catalogs/<catalog>/<catalog>__<config>__summary.md`

Optional flags:
- `--no-validate`
- `--no-plot`
- `--n-maturities`
- `--n-strikes`
