# Catalog Run Summary - `total_variance__default`

## Inputs
- Config selection: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\default.json`
- Selection policy: `first-success`
- Catalog: `total_variance`
- Snapshots count: `10`

## Results
- Success: `3`
- Failed: `7`

| Snapshot | Status | Config | Score | Report | Error |
|---|---|---|---:|---|---|
| `tv_catalog_01_equity_like_left_skew` | success | `default` | `1.030606` | `tv_catalog_01_equity_like_left_skew__default.md` | - |
| `tv_catalog_02_fx_like_symmetric_smile` | failed | - | - | - | `1` |
| `tv_catalog_03_rates_like_low_skew` | failed | - | - | - | `1` |
| `tv_catalog_04_strong_wings` | failed | - | - | - | `1` |
| `tv_catalog_05_weak_wings` | success | `default` | `1.000293` | `tv_catalog_05_weak_wings__default.md` | - |
| `tv_catalog_06_steep_upward_term_structure` | success | `default` | `3.575412` | `tv_catalog_06_steep_upward_term_structure__default.md` | - |
| `tv_catalog_07_inverted_term_structure` | failed | - | - | - | `1` |
| `tv_catalog_08_short_term_event` | failed | - | - | - | `1` |
| `tv_catalog_09_twist` | failed | - | - | - | `1` |
| `tv_catalog_10_high_vol_regime_long_end` | failed | - | - | - | `1` |

