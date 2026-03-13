# Calibration Report - `tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\robust_l1_anchor_strong.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_07_inverted_term_structure.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong`

## Global Metrics
- Number of quotes: `85`
- Objective value: `2.191346e-01`
- Inside bid/ask ratio: `100.00%`
- Bid/ask violation (max / mean): `0.000e+00` / `0.000e+00`
- MAE(mid): `7.386e-06`
- RMSE(mid): `2.553e-05`
- MAE(residual/half-spread): `0.054`
- IV total variation: `3.031e-01`
- IV max second diff: `4.232e+00`
- MAE(iv): `3.464e-04`
- RMSE(iv): `9.196e-04`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=7.386e-06, RMSE=2.553e-05. Erreur en volatilite implicite: MAE=3.464e-04, RMSE=9.196e-04. Rugosite IV (global): TV=3.031e-01, max seconde diff=4.232e+00.

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 15 | 100.00% | 0.000e+00 | 3.792e-05 | 0.297 | 9.548e-02 | 4.232e+00 | 1.000000 | 1.000000 | - | - |
| 0.057534 | 15 | 100.00% | 0.000e+00 | 1.986e-06 | 0.006 | 6.126e-02 | 9.591e-01 | 1.000000 | 1.000000 | - | - |
| 0.123288 | 14 | 100.00% | 0.000e+00 | 4.899e-07 | 0.001 | 4.243e-02 | 6.672e-01 | 1.000000 | 1.000000 | - | - |
| 0.246575 | 14 | 100.00% | 0.000e+00 | 7.655e-07 | 0.001 | 3.731e-02 | 5.798e-01 | 1.000000 | 1.000000 | - | - |
| 0.493151 | 12 | 100.00% | 0.000e+00 | 4.013e-07 | 0.000 | 3.156e-02 | 5.204e-01 | 1.000000 | 1.000000 | - | - |
| 1.000000 | 15 | 100.00% | 0.000e+00 | 4.562e-07 | 0.000 | 3.504e-02 | 4.757e-01 | 1.000000 | 1.000000 | - | - |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong/tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong/tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong/tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong/tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong/tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong/tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_07_inverted_term_structure__robust_l1_anchor_strong`

