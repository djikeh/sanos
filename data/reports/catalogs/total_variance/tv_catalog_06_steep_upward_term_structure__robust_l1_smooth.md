# Calibration Report - `tv_catalog_06_steep_upward_term_structure__robust_l1_smooth`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\robust_l1_smooth.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_06_steep_upward_term_structure.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_06_steep_upward_term_structure__robust_l1_smooth\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_06_steep_upward_term_structure__robust_l1_smooth\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_06_steep_upward_term_structure__robust_l1_smooth`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_06_steep_upward_term_structure__robust_l1_smooth`

## Global Metrics
- Number of quotes: `111`
- Objective value: `1.395225e+00`
- Inside bid/ask ratio: `98.20%`
- Bid/ask violation (max / mean): `7.842e-07` / `7.681e-09`
- MAE(mid): `2.331e-05`
- RMSE(mid): `6.271e-05`
- MAE(residual/half-spread): `0.099`
- IV total variation: `4.004e-01`
- IV max second diff: `3.685e+01`
- MAE(iv): `4.894e-04`
- RMSE(iv): `1.277e-03`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `9.040e-09` / `1.835e-10` / `13`

### Interpretation automatique
La calibration est globalement coherente avec le bid/ask mais quelques quotes sortent du spread. Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=2.331e-05, RMSE=6.271e-05. Erreur en volatilite implicite: MAE=4.894e-04, RMSE=1.277e-03. Rugosite IV (global): TV=4.004e-01, max seconde diff=3.685e+01.

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 9 | 88.89% | 7.842e-07 | 2.726e-05 | 0.434 | 3.906e-02 | 3.685e+01 | 1.000000 | 1.000000 | - | - |
| 0.038356 | 11 | 100.00% | 0.000e+00 | 1.719e-05 | 0.211 | 2.129e-02 | 1.202e+01 | 1.000000 | 1.000000 | - | - |
| 0.082192 | 13 | 92.31% | 6.831e-08 | 3.349e-05 | 0.324 | 3.254e-02 | 4.612e+00 | 1.000000 | 1.000000 | - | - |
| 0.164384 | 13 | 100.00% | 0.000e+00 | 7.408e-06 | 0.012 | 2.552e-02 | 9.071e-01 | 1.000000 | 1.000000 | - | - |
| 0.246575 | 13 | 100.00% | 0.000e+00 | 7.106e-07 | 0.001 | 3.449e-02 | 5.719e-01 | 1.000000 | 1.000000 | - | - |
| 0.493151 | 13 | 100.00% | 0.000e+00 | 6.444e-07 | 0.000 | 4.719e-02 | 6.025e-01 | 1.000000 | 1.000000 | - | - |
| 0.739726 | 13 | 100.00% | 0.000e+00 | 7.205e-07 | 0.000 | 5.619e-02 | 6.418e-01 | 1.000000 | 1.000000 | - | - |
| 1.000000 | 13 | 100.00% | 0.000e+00 | 4.538e-07 | 0.000 | 6.536e-02 | 6.734e-01 | 1.000000 | 1.000000 | - | - |
| 2.000000 | 13 | 100.00% | 0.000e+00 | 1.222e-04 | 0.031 | 7.879e-02 | 8.690e-01 | 1.000000 | 1.000000 | - | - |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__robust_l1_smooth/tv_catalog_06_steep_upward_term_structure__robust_l1_smooth_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__robust_l1_smooth/tv_catalog_06_steep_upward_term_structure__robust_l1_smooth_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__robust_l1_smooth/tv_catalog_06_steep_upward_term_structure__robust_l1_smooth_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__robust_l1_smooth/tv_catalog_06_steep_upward_term_structure__robust_l1_smooth_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__robust_l1_smooth/tv_catalog_06_steep_upward_term_structure__robust_l1_smooth_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__robust_l1_smooth/tv_catalog_06_steep_upward_term_structure__robust_l1_smooth_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__robust_l1_smooth`

