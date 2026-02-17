# Calibration Report - `tv_catalog_10_high_vol_regime_long_end__default`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\default.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_10_high_vol_regime_long_end.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_10_high_vol_regime_long_end__default\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_10_high_vol_regime_long_end__default\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_10_high_vol_regime_long_end__default`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_10_high_vol_regime_long_end__default`

## Global Metrics
- Number of quotes: `112`
- Objective value: `1.464616e+01`
- Inside bid/ask ratio: `93.75%`
- Bid/ask violation (max / mean): `1.376e-02` / `2.977e-04`
- MAE(mid): `1.561e-03`
- RMSE(mid): `3.732e-03`
- MAE(residual/half-spread): `0.263`
- IV total variation: `5.514e-01`
- IV max second diff: `5.328e+01`
- MAE(iv): `1.130e-02`
- RMSE(iv): `4.663e-02`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `1.110e-15` / `1.133e-17` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
Le taux inside bid/ask est faible: la calibration merite un reglage des hyperparametres. Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=1.561e-03, RMSE=3.732e-03. Erreur en volatilite implicite: MAE=1.130e-02, RMSE=4.663e-02. Rugosite IV (global): TV=5.514e-01, max seconde diff=5.328e+01.

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 15 | 93.33% | 1.121e-03 | 1.015e-04 | 0.275 | 3.257e-01 | 5.328e+01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.082192 | 17 | 100.00% | 0.000e+00 | 5.137e-06 | 0.037 | 4.222e-02 | 2.569e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.246575 | 16 | 100.00% | 0.000e+00 | 3.481e-05 | 0.036 | 4.191e-02 | 1.151e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.493151 | 15 | 100.00% | 0.000e+00 | 1.149e-04 | 0.052 | 3.781e-02 | 1.249e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 1.000000 | 16 | 100.00% | 0.000e+00 | 2.874e-04 | 0.072 | 5.408e-02 | 9.667e-01 | 1.000000 | 1.000000 | yes | 2.829e-02 |
| 2.000000 | 17 | 88.24% | 4.107e-03 | 3.536e-03 | 0.558 | 3.084e-02 | 3.584e-01 | 1.000000 | 1.000000 | yes | 9.236e-02 |
| 3.000000 | 16 | 75.00% | 1.376e-02 | 6.642e-03 | 0.795 | 1.877e-02 | 4.336e-01 | 1.000000 | 1.000000 | yes | 5.355e-02 |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__default/tv_catalog_10_high_vol_regime_long_end__default_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__default/tv_catalog_10_high_vol_regime_long_end__default_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__default/tv_catalog_10_high_vol_regime_long_end__default_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__default/tv_catalog_10_high_vol_regime_long_end__default_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__default/tv_catalog_10_high_vol_regime_long_end__default_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__default/tv_catalog_10_high_vol_regime_long_end__default_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__default`

