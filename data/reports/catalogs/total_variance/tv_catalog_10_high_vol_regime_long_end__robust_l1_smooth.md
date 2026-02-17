# Calibration Report - `tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\robust_l1_smooth.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_10_high_vol_regime_long_end.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth`

## Global Metrics
- Number of quotes: `112`
- Objective value: `1.661451e+01`
- Inside bid/ask ratio: `91.07%`
- Bid/ask violation (max / mean): `1.423e-02` / `5.060e-04`
- MAE(mid): `1.832e-03`
- RMSE(mid): `4.398e-03`
- MAE(residual/half-spread): `0.296`
- IV total variation: `4.854e-01`
- IV max second diff: `2.719e+01`
- MAE(iv): `1.186e-02`
- RMSE(iv): `4.607e-02`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `3.331e-16` / `3.399e-18` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
Le taux inside bid/ask est faible: la calibration merite un reglage des hyperparametres. Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=1.832e-03, RMSE=4.398e-03. Erreur en volatilite implicite: MAE=1.186e-02, RMSE=4.607e-02. Rugosite IV (global): TV=4.854e-01, max seconde diff=2.719e+01.

## Oscillation Analysis (Before/After)
- Baseline IV smoothness (no init): TV=`4.776e-01`, max second diff=`2.879e+01`
- Current IV smoothness (linear-density init): TV=`4.854e-01`, max second diff=`2.719e+01`
- Delta (current - baseline): TV=`7.876e-03`, max second diff=`-1.603e+00`
Conclusion: the IV oscillation level decreases after enabling linear-density initialization.

Most improved maturities (delta max second diff):
- T=0.019178, delta=-1.603e+00, baseline strike=0.8600, current strike=0.8600
- T=1.000000, delta=-2.480e-02, baseline strike=0.9000, current strike=0.9333
- T=0.246575, delta=-5.200e-03, baseline strike=0.8600, current strike=0.9000
- T=0.082192, delta=-2.540e-04, baseline strike=0.7800, current strike=0.7800

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 15 | 100.00% | 7.494e-16 | 9.966e-06 | 0.119 | 2.788e-01 | 2.719e+01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.082192 | 17 | 100.00% | 0.000e+00 | 4.894e-06 | 0.025 | 4.132e-02 | 2.565e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.246575 | 16 | 100.00% | 0.000e+00 | 3.602e-05 | 0.048 | 4.191e-02 | 1.112e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.493151 | 15 | 100.00% | 0.000e+00 | 1.153e-04 | 0.052 | 3.776e-02 | 1.270e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 1.000000 | 16 | 100.00% | 0.000e+00 | 4.026e-04 | 0.115 | 4.741e-02 | 7.824e-01 | 1.000000 | 1.000000 | yes | 2.829e-02 |
| 2.000000 | 17 | 64.71% | 1.423e-02 | 5.292e-03 | 0.876 | 1.943e-02 | 6.678e-01 | 1.000000 | 1.000000 | yes | 9.236e-02 |
| 3.000000 | 16 | 75.00% | 1.376e-02 | 6.643e-03 | 0.795 | 1.878e-02 | 4.337e-01 | 1.000000 | 1.000000 | yes | 5.355e-02 |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth/tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth/tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth/tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth/tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth/tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth/tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_10_high_vol_regime_long_end__robust_l1_smooth`

