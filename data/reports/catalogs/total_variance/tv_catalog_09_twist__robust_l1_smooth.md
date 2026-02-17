# Calibration Report - `tv_catalog_09_twist__robust_l1_smooth`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\robust_l1_smooth.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_09_twist.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_09_twist__robust_l1_smooth\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_09_twist__robust_l1_smooth\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_09_twist__robust_l1_smooth`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_09_twist__robust_l1_smooth`

## Global Metrics
- Number of quotes: `315`
- Objective value: `7.342761e+00`
- Inside bid/ask ratio: `99.68%`
- Bid/ask violation (max / mean): `5.400e-12` / `1.714e-14`
- MAE(mid): `8.271e-05`
- RMSE(mid): `1.849e-04`
- MAE(residual/half-spread): `0.054`
- IV total variation: `5.185e-01`
- IV max second diff: `3.975e+00`
- MAE(iv): `3.673e-04`
- RMSE(iv): `8.016e-04`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=8.271e-05, RMSE=1.849e-04. Erreur en volatilite implicite: MAE=3.673e-04, RMSE=8.016e-04. Rugosite IV (global): TV=5.185e-01, max seconde diff=3.975e+00.

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 35 | 97.14% | 5.400e-12 | 1.216e-05 | 0.095 | 1.468e-01 | 3.975e+00 | 1.000000 | 1.000000 | yes | 1.511e-01 |
| 0.038356 | 35 | 100.00% | 0.000e+00 | 1.926e-05 | 0.072 | 6.920e-02 | 2.202e+00 | 1.000000 | 1.000000 | yes | 2.919e-01 |
| 0.082192 | 35 | 100.00% | 0.000e+00 | 3.058e-05 | 0.050 | 4.458e-02 | 1.391e+00 | 1.000000 | 1.000000 | yes | 3.381e-01 |
| 0.164384 | 35 | 100.00% | 0.000e+00 | 3.892e-05 | 0.041 | 4.015e-02 | 1.032e+00 | 1.000000 | 1.000000 | yes | 5.263e-01 |
| 0.246575 | 35 | 100.00% | 0.000e+00 | 4.276e-05 | 0.037 | 3.724e-02 | 8.206e-01 | 1.000000 | 1.000000 | yes | 2.212e-01 |
| 0.493151 | 35 | 100.00% | 0.000e+00 | 8.266e-05 | 0.045 | 4.246e-02 | 8.849e-01 | 1.000000 | 1.000000 | yes | 9.731e-01 |
| 0.739726 | 35 | 100.00% | 0.000e+00 | 1.211e-04 | 0.050 | 4.479e-02 | 1.001e+00 | 1.000000 | 1.000000 | yes | 1.009e+00 |
| 1.000000 | 35 | 100.00% | 0.000e+00 | 1.399e-04 | 0.048 | 4.686e-02 | 5.957e-01 | 1.000000 | 1.000000 | yes | 1.829e+00 |
| 2.000000 | 35 | 100.00% | 0.000e+00 | 2.569e-04 | 0.053 | 4.644e-02 | 5.857e-01 | 1.000000 | 1.000000 | yes | 1.134e+00 |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_09_twist__robust_l1_smooth/tv_catalog_09_twist__robust_l1_smooth_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_09_twist__robust_l1_smooth/tv_catalog_09_twist__robust_l1_smooth_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_09_twist__robust_l1_smooth/tv_catalog_09_twist__robust_l1_smooth_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_09_twist__robust_l1_smooth/tv_catalog_09_twist__robust_l1_smooth_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_09_twist__robust_l1_smooth/tv_catalog_09_twist__robust_l1_smooth_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_09_twist__robust_l1_smooth/tv_catalog_09_twist__robust_l1_smooth_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_09_twist__robust_l1_smooth`

