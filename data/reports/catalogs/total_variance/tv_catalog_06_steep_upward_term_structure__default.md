# Calibration Report - `tv_catalog_06_steep_upward_term_structure__default`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\default.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_06_steep_upward_term_structure.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_06_steep_upward_term_structure__default\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_06_steep_upward_term_structure__default\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_06_steep_upward_term_structure__default`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_06_steep_upward_term_structure__default`

## Global Metrics
- Number of quotes: `111`
- Objective value: `5.463908e-01`
- Inside bid/ask ratio: `100.00%`
- Bid/ask violation (max / mean): `0.000e+00` / `0.000e+00`
- MAE(mid): `1.187e-05`
- RMSE(mid): `7.494e-05`
- MAE(residual/half-spread): `0.010`
- IV total variation: `3.492e-01`
- IV max second diff: `2.986e+01`
- MAE(iv): `7.125e-05`
- RMSE(iv): `3.542e-04`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `1.789e-07` / `9.767e-10` / `13`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=1.187e-05, RMSE=7.494e-05. Erreur en volatilite implicite: MAE=7.125e-05, RMSE=3.542e-04. Rugosite IV (global): TV=3.492e-01, max seconde diff=2.986e+01.

## Oscillation Analysis (Before/After)
- Baseline IV smoothness (no init): TV=`3.492e-01`, max second diff=`2.986e+01`
- Current IV smoothness (linear-density init): TV=`3.492e-01`, max second diff=`2.986e+01`
- Delta (current - baseline): TV=`-1.317e-06`, max second diff=`4.385e-11`
Conclusion: oscillation did not improve globally; inspect per-maturity rows below.

Most improved maturities (delta max second diff):
- T=0.038356, delta=-6.687e-10, baseline strike=0.9900, current strike=0.9900
- T=0.739726, delta=-1.497e-11, baseline strike=0.9200, current strike=0.9200
- T=0.493151, delta=-6.870e-12, baseline strike=0.8800, current strike=0.8800
- T=2.000000, delta=-1.147e-12, baseline strike=0.8800, current strike=0.8800

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 9 | 100.00% | 0.000e+00 | 1.606e-05 | 0.062 | 1.267e-02 | 2.986e+01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.038356 | 11 | 100.00% | 0.000e+00 | 4.079e-06 | 0.017 | 1.143e-02 | 2.415e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.082192 | 13 | 100.00% | 0.000e+00 | 3.697e-07 | 0.001 | 1.872e-02 | 7.091e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.164384 | 13 | 100.00% | 0.000e+00 | 1.466e-07 | 0.000 | 2.536e-02 | 5.661e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.246575 | 13 | 100.00% | 0.000e+00 | 6.751e-09 | 0.000 | 3.447e-02 | 5.648e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.493151 | 13 | 100.00% | 0.000e+00 | 1.692e-08 | 0.000 | 4.718e-02 | 6.016e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.739726 | 13 | 100.00% | 0.000e+00 | 5.745e-09 | 0.000 | 5.618e-02 | 6.362e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 1.000000 | 13 | 100.00% | 0.000e+00 | 1.731e-08 | 0.000 | 6.536e-02 | 6.742e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 2.000000 | 13 | 100.00% | 0.000e+00 | 8.622e-05 | 0.025 | 7.782e-02 | 7.969e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__default/tv_catalog_06_steep_upward_term_structure__default_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__default/tv_catalog_06_steep_upward_term_structure__default_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__default/tv_catalog_06_steep_upward_term_structure__default_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__default/tv_catalog_06_steep_upward_term_structure__default_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__default/tv_catalog_06_steep_upward_term_structure__default_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__default/tv_catalog_06_steep_upward_term_structure__default_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_06_steep_upward_term_structure__default`

