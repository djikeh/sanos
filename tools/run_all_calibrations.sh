#!/usr/bin/env bash
# Run all 5 configs against all 3 snapshot catalogs (150 calibrations total).
# Uses the compiled release binary directly for speed.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CLI="$ROOT/target/release/sanos"

if [ ! -f "$CLI" ] && [ -f "$CLI.exe" ]; then
  CLI="$CLI.exe"
fi

CONFIGS=(tight_spread zero_spread_vega strong_wings sparse_robust extreme_stress)
CATALOGS=(sabr svi_raw total_variance)

total=$((${#CONFIGS[@]} * ${#CATALOGS[@]}))
i=0

for cfg in "${CONFIGS[@]}"; do
  for cat in "${CATALOGS[@]}"; do
    i=$((i + 1))
    SNAP_DIR="$ROOT/data/snapshots/catalogs/$cat"
    CFG_FILE="$ROOT/data/configs/${cfg}.json"
    OUT_ROOT="$ROOT/data/surfaces/catalogs/${cat}"

    echo "============================================================"
    echo "[$i/$total] Config: $cfg | Catalog: $cat"
    echo "============================================================"

    for snapshot in "$SNAP_DIR"/*.json; do
      snap_name=$(basename "$snapshot" .snapshot.json)
      # strip .json too if it doesn't have .snapshot.json suffix
      snap_name=${snap_name%.json}
      run_id="${snap_name}__${cfg}"
      out_dir="$OUT_ROOT/$run_id"

      mkdir -p "$out_dir"

      echo "  -> $snap_name with $cfg"
      "$CLI" calibrate \
        --snapshot "$snapshot" \
        --config "$CFG_FILE" \
        --out "$out_dir" \
        --n-maturities 41 \
        --n-strikes 81 2>&1 || {
        echo "  !! FAILED: $snap_name with $cfg"
        continue
      }
    done
  done
done

echo ""
echo "All calibrations complete."
