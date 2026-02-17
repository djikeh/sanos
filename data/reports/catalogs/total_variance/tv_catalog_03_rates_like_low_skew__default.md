# Calibration Report - `tv_catalog_03_rates_like_low_skew__default`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\default.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_03_rates_like_low_skew.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_03_rates_like_low_skew__default\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_03_rates_like_low_skew__default\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_03_rates_like_low_skew__default`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_03_rates_like_low_skew__default`

## Global Metrics
- Number of quotes: `175`
- Objective value: `9.225329e+07`
- Inside bid/ask ratio: `51.43%`
- Bid/ask violation (max / mean): `1.385e-05` / `5.272e-07`
- MAE(mid): `5.272e-07`
- RMSE(mid): `1.738e-06`
- MAE(residual/half-spread): `1054325.430`
- IV total variation: `4.422e-02`
- IV max second diff: `7.171e-01`
- MAE(iv): `3.878e-06`
- RMSE(iv): `1.560e-05`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
Le taux inside bid/ask est faible: la calibration merite un reglage des hyperparametres. Les residus normalises sont eleves face aux spreads: la robustesse locale est a ameliorer. Erreur en prix: MAE=5.272e-07, RMSE=1.738e-06. Erreur en volatilite implicite: MAE=3.878e-06, RMSE=1.560e-05. Rugosite IV (global): TV=4.422e-02, max seconde diff=7.171e-01.

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 25 | 88.00% | 2.112e-11 | 1.963e-12 | 3.926 | 7.269e-03 | 5.492e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.038356 | 25 | 92.00% | 7.721e-11 | 5.406e-12 | 10.812 | 6.815e-03 | 3.279e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.082192 | 25 | 72.00% | 1.263e-08 | 1.133e-09 | 2265.992 | 6.419e-03 | 3.090e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.246575 | 25 | 36.00% | 1.593e-06 | 4.050e-07 | 809941.757 | 6.105e-03 | 5.321e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.493151 | 25 | 24.00% | 1.385e-05 | 2.808e-06 | 5616957.912 | 6.112e-03 | 7.171e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 1.000000 | 25 | 24.00% | 7.182e-07 | 2.599e-07 | 519751.471 | 5.808e-03 | 2.772e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 2.000000 | 25 | 24.00% | 1.145e-06 | 2.157e-07 | 431346.140 | 5.693e-03 | 2.641e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_03_rates_like_low_skew__default/tv_catalog_03_rates_like_low_skew__default_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_03_rates_like_low_skew__default/tv_catalog_03_rates_like_low_skew__default_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_03_rates_like_low_skew__default/tv_catalog_03_rates_like_low_skew__default_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_03_rates_like_low_skew__default/tv_catalog_03_rates_like_low_skew__default_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_03_rates_like_low_skew__default/tv_catalog_03_rates_like_low_skew__default_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_03_rates_like_low_skew__default/tv_catalog_03_rates_like_low_skew__default_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_03_rates_like_low_skew__default`

