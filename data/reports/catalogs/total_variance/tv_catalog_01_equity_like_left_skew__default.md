# Calibration Report - `tv_catalog_01_equity_like_left_skew__default`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\default.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_01_equity_like_left_skew.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_01_equity_like_left_skew__default\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_01_equity_like_left_skew__default\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_01_equity_like_left_skew__default`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_01_equity_like_left_skew__default`

## Global Metrics
- Number of quotes: `175`
- Objective value: `1.366577e-01`
- Inside bid/ask ratio: `100.00%`
- Bid/ask violation (max / mean): `0.000e+00` / `0.000e+00`
- MAE(mid): `3.188e-06`
- RMSE(mid): `2.032e-05`
- MAE(residual/half-spread): `0.002`
- IV total variation: `7.702e-01`
- IV max second diff: `1.359e+00`
- MAE(iv): `1.605e-05`
- RMSE(iv): `1.168e-04`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=3.188e-06, RMSE=2.032e-05. Erreur en volatilite implicite: MAE=1.605e-05, RMSE=1.168e-04. Rugosite IV (global): TV=7.702e-01, max seconde diff=1.359e+00.

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 35 | 100.00% | 0.000e+00 | 7.152e-11 | 0.000 | 2.007e-01 | 1.359e+00 | 1.000000 | 1.000000 | - | - |
| 0.082192 | 35 | 100.00% | 0.000e+00 | 5.248e-09 | 0.000 | 1.637e-01 | 1.104e+00 | 1.000000 | 1.000000 | - | - |
| 0.246575 | 35 | 100.00% | 0.000e+00 | 8.032e-09 | 0.000 | 1.451e-01 | 9.777e-01 | 1.000000 | 1.000000 | - | - |
| 0.493151 | 35 | 100.00% | 0.000e+00 | 1.929e-08 | 0.000 | 1.355e-01 | 9.118e-01 | 1.000000 | 1.000000 | - | - |
| 1.000000 | 35 | 100.00% | 0.000e+00 | 1.591e-05 | 0.008 | 1.252e-01 | 8.411e-01 | 1.000000 | 1.000000 | - | - |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_01_equity_like_left_skew__default/tv_catalog_01_equity_like_left_skew__default_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_01_equity_like_left_skew__default/tv_catalog_01_equity_like_left_skew__default_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_01_equity_like_left_skew__default/tv_catalog_01_equity_like_left_skew__default_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_01_equity_like_left_skew__default/tv_catalog_01_equity_like_left_skew__default_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_01_equity_like_left_skew__default/tv_catalog_01_equity_like_left_skew__default_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_01_equity_like_left_skew__default/tv_catalog_01_equity_like_left_skew__default_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_01_equity_like_left_skew__default`

