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
- Objective value: `1.775910e+01`
- Inside bid/ask ratio: `90.18%`
- Bid/ask violation (max / mean): `1.423e-02` / `5.159e-04`
- MAE(mid): `1.844e-03`
- RMSE(mid): `4.400e-03`
- MAE(residual/half-spread): `0.317`
- IV total variation: `5.127e-01`
- IV max second diff: `5.328e+01`
- MAE(iv): `1.164e-02`
- RMSE(iv): `4.453e-02`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `2.220e-16` / `2.266e-18` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
Le taux inside bid/ask est faible: la calibration merite un reglage des hyperparametres. Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=1.844e-03, RMSE=4.400e-03. Erreur en volatilite implicite: MAE=1.164e-02, RMSE=4.453e-02. Rugosite IV (global): TV=5.127e-01, max seconde diff=5.328e+01.

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 15 | 93.33% | 1.121e-03 | 1.015e-04 | 0.274 | 3.062e-01 | 5.328e+01 | 1.000000 | 1.000000 | - | - |
| 0.082192 | 17 | 100.00% | 0.000e+00 | 5.137e-06 | 0.028 | 4.148e-02 | 2.566e+00 | 1.000000 | 1.000000 | - | - |
| 0.246575 | 16 | 100.00% | 0.000e+00 | 3.578e-05 | 0.047 | 4.191e-02 | 1.127e+00 | 1.000000 | 1.000000 | - | - |
| 0.493151 | 15 | 100.00% | 0.000e+00 | 1.149e-04 | 0.052 | 3.781e-02 | 1.249e+00 | 1.000000 | 1.000000 | - | - |
| 1.000000 | 16 | 100.00% | 0.000e+00 | 3.991e-04 | 0.114 | 4.708e-02 | 7.499e-01 | 1.000000 | 1.000000 | - | - |
| 2.000000 | 17 | 64.71% | 1.423e-02 | 5.292e-03 | 0.876 | 1.943e-02 | 6.678e-01 | 1.000000 | 1.000000 | - | - |
| 3.000000 | 16 | 75.00% | 1.376e-02 | 6.642e-03 | 0.795 | 1.877e-02 | 4.336e-01 | 1.000000 | 1.000000 | - | - |

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

