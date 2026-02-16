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
- `surface.json` (dense and pretty-printed)
- `diagnostics.json`

## Python orchestrator (`sanos run`)

The top-level orchestrator is Python and does not create Rust dependencies on `tools/`.

```bash
python tools/sanos.py run --config default --snapshot my_snapshot
```

Shortcuts:
- Windows (from repo root): `sanos.cmd run --config default --snapshot my_snapshot`
- Linux/macOS (from repo root): `./sanos run --config default --snapshot my_snapshot`

It resolves:
- `data/configs/default.json`
- `data/snapshots/my_snapshot.json`

Then it:
1. Validates the snapshot.
2. Runs calibration via Rust CLI.
3. Writes JSON outputs in `data/surfaces/<snapshot>__<config>/`.
4. Generates calibration quality plots in `data/images/<snapshot>__<config>/`.

Optional flags:
- `--no-validate`
- `--no-plot`
- `--n-maturities`
- `--n-strikes`
