# Catalog Run Summary - `total_variance__catalog_total_variance_robust`

## Inputs
- Config selection: `C:\Users\djike\Dev\Repos\paper\sanos\data\config_catalogs\total_variance_robust.json`
- Selection policy: `first-success`
- Catalog: `total_variance`
- Snapshots count: `10`

## Results
- Success: `3`
- Failed: `7`

| Snapshot | Status | Config | Score | Report | Error |
|---|---|---|---:|---|---|
| `tv_catalog_01_equity_like_left_skew` | success | `default` | `2.151076` | `tv_catalog_01_equity_like_left_skew__default.md` | - |
| `tv_catalog_02_fx_like_symmetric_smile` | failed | - | - | - | `all config attempts failed (robust_hinge_strict: 1; robust_l1_anchor_strong: 1; default: 1; robust_l1_smooth: 1)` |
| `tv_catalog_03_rates_like_low_skew` | failed | - | - | - | `all config attempts failed (robust_l1_anchor_strong: 1; default: 1; robust_hinge_strict: 1; robust_l1_smooth: 1)` |
| `tv_catalog_04_strong_wings` | failed | - | - | - | `all config attempts failed (default: 1; robust_hinge_strict: 1; robust_l1_anchor_strong: 1; robust_l1_smooth: 1)` |
| `tv_catalog_05_weak_wings` | failed | - | - | - | `all config attempts failed (default: 1; robust_l1_anchor_strong: 1; robust_hinge_strict: 1; robust_l1_smooth: 1)` |
| `tv_catalog_06_steep_upward_term_structure` | success | `robust_l1_smooth` | `13000006.302916` | `tv_catalog_06_steep_upward_term_structure__robust_l1_smooth.md` | - |
| `tv_catalog_07_inverted_term_structure` | success | `robust_l1_anchor_strong` | `2.153953` | `tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong.md` | - |
| `tv_catalog_08_short_term_event` | failed | - | - | - | `all config attempts failed (robust_l1_anchor_strong: 1; default: 1; robust_hinge_strict: 1; robust_l1_smooth: 1)` |
| `tv_catalog_09_twist` | failed | - | - | - | `all config attempts failed (robust_hinge_strict: 1; robust_l1_anchor_strong: 1; default: 1; robust_l1_smooth: 1)` |
| `tv_catalog_10_high_vol_regime_long_end` | failed | - | - | - | `all config attempts failed (robust_l1_smooth: 1; default: 1; robust_l1_anchor_strong: 1; robust_hinge_strict: 1)` |

