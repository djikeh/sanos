# Calibration Report - `tv_catalog_08_short_term_event__robust_l1_anchor_strong`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\robust_l1_anchor_strong.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_08_short_term_event.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_08_short_term_event__robust_l1_anchor_strong\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_08_short_term_event__robust_l1_anchor_strong\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_08_short_term_event__robust_l1_anchor_strong`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_08_short_term_event__robust_l1_anchor_strong`

## Global Metrics
- Number of quotes: `99`
- Objective value: `1.424276e+10`
- Inside bid/ask ratio: `21.21%`
- Bid/ask violation (max / mean): `2.478e-03` / `1.439e-04`
- MAE(mid): `1.439e-04`
- RMSE(mid): `3.575e-04`
- MAE(residual/half-spread): `287732470.426`
- IV total variation: `3.799e-01`
- IV max second diff: `4.788e+00`
- MAE(iv): `1.098e-03`
- RMSE(iv): `1.944e-03`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
Le taux inside bid/ask est faible: la calibration merite un reglage des hyperparametres. Les residus normalises sont eleves face aux spreads: la robustesse locale est a ameliorer. Erreur en prix: MAE=1.439e-04, RMSE=3.575e-04. Erreur en volatilite implicite: MAE=1.098e-03, RMSE=1.944e-03. Rugosite IV (global): TV=3.799e-01, max seconde diff=4.788e+00.

## Oscillation Analysis (Before/After)
- Baseline IV smoothness (no init): TV=`3.799e-01`, max second diff=`4.788e+00`
- Current IV smoothness (linear-density init): TV=`3.799e-01`, max second diff=`4.788e+00`
- Delta (current - baseline): TV=`-3.302e-07`, max second diff=`2.529e-05`
Conclusion: oscillation did not improve globally; inspect per-maturity rows below.

Most improved maturities (delta max second diff):
- T=0.027397, delta=-5.454e-08, baseline strike=1.0100, current strike=1.0100
- T=1.000000, delta=-9.257e-09, baseline strike=0.9700, current strike=0.9700
- T=0.249315, delta=-1.084e-12, baseline strike=0.9700, current strike=0.9700
- T=2.000000, delta=7.455e-11, baseline strike=0.7200, current strike=0.7200

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 15 | 20.00% | 8.224e-05 | 1.671e-05 | 33423819.557 | 7.535e-02 | 3.648e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.027397 | 15 | 20.00% | 1.400e-04 | 1.850e-05 | 37004816.042 | 7.203e-02 | 3.440e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.038356 | 15 | 6.67% | 2.204e-04 | 3.236e-05 | 64727718.072 | 7.550e-02 | 4.788e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.246575 | 15 | 26.67% | 5.441e-04 | 1.864e-04 | 372749335.040 | 4.227e-02 | 1.273e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.249315 | 13 | 46.15% | 4.609e-04 | 7.721e-05 | 154424931.278 | 3.894e-02 | 9.181e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 1.000000 | 13 | 0.00% | 8.584e-04 | 2.002e-04 | 400382833.624 | 3.516e-02 | 8.444e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 2.000000 | 13 | 30.77% | 2.478e-03 | 5.252e-04 | 1050340638.287 | 4.064e-02 | 7.996e-01 | 1.000000 | 1.000000 | yes | 1.497e-03 |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_08_short_term_event__robust_l1_anchor_strong/tv_catalog_08_short_term_event__robust_l1_anchor_strong_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_08_short_term_event__robust_l1_anchor_strong/tv_catalog_08_short_term_event__robust_l1_anchor_strong_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_08_short_term_event__robust_l1_anchor_strong/tv_catalog_08_short_term_event__robust_l1_anchor_strong_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_08_short_term_event__robust_l1_anchor_strong/tv_catalog_08_short_term_event__robust_l1_anchor_strong_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_08_short_term_event__robust_l1_anchor_strong/tv_catalog_08_short_term_event__robust_l1_anchor_strong_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_08_short_term_event__robust_l1_anchor_strong/tv_catalog_08_short_term_event__robust_l1_anchor_strong_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_08_short_term_event__robust_l1_anchor_strong`

