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
- Objective value: `4.132011e-01`
- Inside bid/ask ratio: `100.00%`
- Bid/ask violation (max / mean): `0.000e+00` / `0.000e+00`
- MAE(mid): `1.118e-05`
- RMSE(mid): `7.586e-05`
- MAE(residual/half-spread): `0.006`
- IV total variation: `3.467e-01`
- IV max second diff: `3.010e+01`
- MAE(iv): `5.240e-05`
- RMSE(iv): `3.105e-04`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `3.997e-14` / `8.755e-16` / `0`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=1.118e-05, RMSE=7.586e-05. Erreur en volatilite implicite: MAE=5.240e-05, RMSE=3.105e-04. Rugosite IV (global): TV=3.467e-01, max seconde diff=3.010e+01.

## Oscillation Analysis (Before/After)
- Baseline IV smoothness (no init): TV=`3.468e-01`, max second diff=`3.010e+01`
- Current IV smoothness (linear-density init): TV=`3.467e-01`, max second diff=`3.010e+01`
- Delta (current - baseline): TV=`-1.398e-04`, max second diff=`1.488e-10`
Conclusion: oscillation did not improve globally; inspect per-maturity rows below.

Most improved maturities (delta max second diff):
- T=0.038356, delta=-5.705e-11, baseline strike=0.9500, current strike=0.9500
- T=0.019178, delta=1.488e-10, baseline strike=1.0000, current strike=1.0000
- T=0.246575, delta=1.729e-10, baseline strike=0.9200, current strike=0.9200
- T=1.000000, delta=8.064e-09, baseline strike=0.7200, current strike=0.7200

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 9 | 100.00% | 0.000e+00 | 1.170e-05 | 0.042 | 1.061e-02 | 3.010e+01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.038356 | 11 | 100.00% | 0.000e+00 | 9.537e-18 | 0.000 | 1.115e-02 | 7.966e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.082192 | 13 | 100.00% | 0.000e+00 | 4.294e-08 | 0.000 | 1.872e-02 | 6.311e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.164384 | 13 | 100.00% | 0.000e+00 | 2.705e-07 | 0.000 | 2.536e-02 | 5.948e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.246575 | 13 | 100.00% | 0.000e+00 | 5.591e-09 | 0.000 | 3.447e-02 | 5.648e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.493151 | 13 | 100.00% | 0.000e+00 | 7.688e-08 | 0.000 | 4.718e-02 | 6.018e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.739726 | 13 | 100.00% | 0.000e+00 | 1.801e-08 | 0.000 | 5.618e-02 | 6.363e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 1.000000 | 13 | 100.00% | 0.000e+00 | 1.347e-07 | 0.000 | 6.536e-02 | 6.742e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 2.000000 | 13 | 100.00% | 0.000e+00 | 8.684e-05 | 0.025 | 7.763e-02 | 8.047e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |

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

