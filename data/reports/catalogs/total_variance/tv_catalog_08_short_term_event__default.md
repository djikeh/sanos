# Calibration Report - `tv_catalog_08_short_term_event__default`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\default.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_08_short_term_event.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_08_short_term_event__default\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_08_short_term_event__default\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_08_short_term_event__default`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_08_short_term_event__default`

## Global Metrics
- Number of quotes: `99`
- Objective value: `1.556951e+10`
- Inside bid/ask ratio: `13.13%`
- Bid/ask violation (max / mean): `2.478e-03` / `1.573e-04`
- MAE(mid): `1.573e-04`
- RMSE(mid): `3.658e-04`
- MAE(residual/half-spread): `314535532.430`
- IV total variation: `5.324e-01`
- IV max second diff: `1.331e+01`
- MAE(iv): `3.369e-03`
- RMSE(iv): `1.626e-02`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `2.195e-07` / `7.159e-10` / `2`

### Interpretation automatique
Le taux inside bid/ask est faible: la calibration merite un reglage des hyperparametres. Les residus normalises sont eleves face aux spreads: la robustesse locale est a ameliorer. Erreur en prix: MAE=1.573e-04, RMSE=3.658e-04. Erreur en volatilite implicite: MAE=3.369e-03, RMSE=1.626e-02. Rugosite IV (global): TV=5.324e-01, max seconde diff=1.331e+01.

## Oscillation Analysis (Before/After)
- Baseline IV smoothness (no init): TV=`3.799e-01`, max second diff=`4.788e+00`
- Current IV smoothness (linear-density init): TV=`5.324e-01`, max second diff=`1.331e+01`
- Delta (current - baseline): TV=`1.525e-01`, max second diff=`8.525e+00`
Conclusion: oscillation did not improve globally; inspect per-maturity rows below.

Most improved maturities (delta max second diff):
- T=0.246575, delta=-5.450e-04, baseline strike=0.8200, current strike=0.8200
- T=1.000000, delta=-1.102e-08, baseline strike=0.9700, current strike=0.9700
- T=2.000000, delta=-3.109e-11, baseline strike=0.7200, current strike=0.7200
- T=0.249315, delta=3.350e-03, baseline strike=0.9700, current strike=0.9700

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 15 | 0.00% | 7.894e-05 | 1.415e-05 | 28304908.190 | 8.085e-02 | 5.527e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.027397 | 15 | 0.00% | 1.593e-04 | 2.455e-05 | 49091639.420 | 7.386e-02 | 3.558e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.038356 | 15 | 0.00% | 4.936e-04 | 1.109e-04 | 221871920.056 | 2.175e-01 | 1.331e+01 | 0.999999 | 0.999999 | no | 0.000e+00 |
| 0.246575 | 15 | 13.33% | 5.569e-04 | 1.930e-04 | 386040081.400 | 4.440e-02 | 1.272e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.249315 | 13 | 53.85% | 4.662e-04 | 7.692e-05 | 153844927.998 | 3.999e-02 | 9.214e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 1.000000 | 13 | 0.00% | 8.584e-04 | 2.002e-04 | 400382854.840 | 3.516e-02 | 8.444e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 2.000000 | 13 | 30.77% | 2.478e-03 | 5.252e-04 | 1050340638.282 | 4.064e-02 | 7.996e-01 | 1.000000 | 1.000000 | yes | 1.497e-03 |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_08_short_term_event__default/tv_catalog_08_short_term_event__default_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_08_short_term_event__default/tv_catalog_08_short_term_event__default_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_08_short_term_event__default/tv_catalog_08_short_term_event__default_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_08_short_term_event__default/tv_catalog_08_short_term_event__default_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_08_short_term_event__default/tv_catalog_08_short_term_event__default_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_08_short_term_event__default/tv_catalog_08_short_term_event__default_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_08_short_term_event__default`

