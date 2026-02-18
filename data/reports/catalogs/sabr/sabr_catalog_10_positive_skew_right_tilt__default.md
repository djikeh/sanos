# Calibration Report - `sabr_catalog_10_positive_skew_right_tilt__default`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\default.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\sabr\sabr_catalog_10_positive_skew_right_tilt.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\sabr\sabr_catalog_10_positive_skew_right_tilt__default\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\sabr\sabr_catalog_10_positive_skew_right_tilt__default\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\sabr\sabr_catalog_10_positive_skew_right_tilt__default`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\sabr\sabr_catalog_10_positive_skew_right_tilt__default`

## Global Metrics
- Number of quotes: `155`
- Objective value: `3.229486e-03`
- Inside bid/ask ratio: `100.00%`
- Bid/ask violation (max / mean): `2.639e-15` / `2.920e-17`
- MAE(mid): `1.887e-08`
- RMSE(mid): `5.083e-08`
- MAE(residual/half-spread): `0.007`
- IV total variation: `2.852e-01`
- IV max second diff: `5.066e+01`
- MAE(iv): `2.150e-04`
- RMSE(iv): `1.623e-03`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=1.887e-08, RMSE=5.083e-08. Erreur en volatilite implicite: MAE=2.150e-04, RMSE=1.623e-03. Rugosite IV (global): TV=2.852e-01, max seconde diff=5.066e+01.

## Oscillation Analysis (Before/After)
- Baseline IV smoothness (no init): TV=`2.852e-01`, max second diff=`5.066e+01`
- Current IV smoothness (linear-density init): TV=`2.852e-01`, max second diff=`5.066e+01`
- Delta (current - baseline): TV=`0.000e+00`, max second diff=`0.000e+00`
Conclusion: oscillation did not improve globally; inspect per-maturity rows below.

Most improved maturities (delta max second diff):
- T=0.019178, delta=0.000e+00, baseline strike=0.7700, current strike=0.7700
- T=0.082192, delta=0.000e+00, baseline strike=0.7700, current strike=0.7700
- T=0.246575, delta=0.000e+00, baseline strike=0.7700, current strike=0.7700
- T=0.493151, delta=0.000e+00, baseline strike=0.7700, current strike=0.7700

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 31 | 100.00% | 2.639e-15 | 1.032e-10 | 0.033 | 8.912e-02 | 5.066e+01 | 1.000000 | 1.000000 | - | - |
| 0.082192 | 31 | 100.00% | 0.000e+00 | 6.633e-10 | 0.000 | 6.194e-02 | 1.144e+00 | 1.000000 | 1.000000 | - | - |
| 0.246575 | 31 | 100.00% | 0.000e+00 | 6.894e-08 | 0.000 | 5.226e-02 | 9.566e-01 | 1.000000 | 1.000000 | - | - |
| 0.493151 | 31 | 100.00% | 0.000e+00 | 8.498e-09 | 0.000 | 4.400e-02 | 8.238e-01 | 1.000000 | 1.000000 | - | - |
| 1.000000 | 31 | 100.00% | 0.000e+00 | 1.616e-08 | 0.000 | 3.791e-02 | 7.240e-01 | 1.000000 | 1.000000 | - | - |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/sabr/sabr_catalog_10_positive_skew_right_tilt__default/sabr_catalog_10_positive_skew_right_tilt__default_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/sabr/sabr_catalog_10_positive_skew_right_tilt__default/sabr_catalog_10_positive_skew_right_tilt__default_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/sabr/sabr_catalog_10_positive_skew_right_tilt__default/sabr_catalog_10_positive_skew_right_tilt__default_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/sabr/sabr_catalog_10_positive_skew_right_tilt__default/sabr_catalog_10_positive_skew_right_tilt__default_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/sabr/sabr_catalog_10_positive_skew_right_tilt__default/sabr_catalog_10_positive_skew_right_tilt__default_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/sabr/sabr_catalog_10_positive_skew_right_tilt__default/sabr_catalog_10_positive_skew_right_tilt__default_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/sabr/sabr_catalog_10_positive_skew_right_tilt__default`

