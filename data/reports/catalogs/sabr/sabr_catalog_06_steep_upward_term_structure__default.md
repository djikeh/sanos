# Calibration Report - `sabr_catalog_06_steep_upward_term_structure__default`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\default.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\sabr\sabr_catalog_06_steep_upward_term_structure.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\sabr\sabr_catalog_06_steep_upward_term_structure__default\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\sabr\sabr_catalog_06_steep_upward_term_structure__default\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\sabr\sabr_catalog_06_steep_upward_term_structure__default`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\sabr\sabr_catalog_06_steep_upward_term_structure__default`

## Global Metrics
- Number of quotes: `111`
- Objective value: `4.579110e-01`
- Inside bid/ask ratio: `100.00%`
- Bid/ask violation (max / mean): `0.000e+00` / `0.000e+00`
- MAE(mid): `2.217e-06`
- RMSE(mid): `1.152e-05`
- MAE(residual/half-spread): `0.008`
- IV total variation: `4.357e-01`
- IV max second diff: `4.340e+01`
- MAE(iv): `4.564e-05`
- RMSE(iv): `2.482e-04`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=2.217e-06, RMSE=1.152e-05. Erreur en volatilite implicite: MAE=4.564e-05, RMSE=2.482e-04. Rugosite IV (global): TV=4.357e-01, max seconde diff=4.340e+01.

## Oscillation Analysis (Before/After)
- Baseline IV smoothness (no init): TV=`4.187e-01`, max second diff=`4.340e+01`
- Current IV smoothness (linear-density init): TV=`4.357e-01`, max second diff=`4.340e+01`
- Delta (current - baseline): TV=`1.696e-02`, max second diff=`5.691e-12`
Conclusion: oscillation did not improve globally; inspect per-maturity rows below.

Most improved maturities (delta max second diff):
- T=0.164384, delta=-4.439e-07, baseline strike=0.9000, current strike=0.9000
- T=0.082192, delta=-1.776e-11, baseline strike=1.0000, current strike=1.0000
- T=0.246575, delta=-4.872e-12, baseline strike=0.8000, current strike=0.8000
- T=0.019178, delta=5.691e-12, baseline strike=1.0000, current strike=1.0000

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 9 | 100.00% | 0.000e+00 | 2.092e-05 | 0.081 | 1.723e-02 | 4.340e+01 | 1.000000 | 1.000000 | - | - |
| 0.038356 | 11 | 100.00% | 0.000e+00 | 4.390e-06 | 0.016 | 1.960e-02 | 2.492e+00 | 1.000000 | 1.000000 | - | - |
| 0.082192 | 13 | 100.00% | 0.000e+00 | 4.740e-07 | 0.001 | 3.034e-02 | 7.505e-01 | 1.000000 | 1.000000 | - | - |
| 0.164384 | 13 | 100.00% | 0.000e+00 | 4.995e-10 | 0.000 | 3.911e-02 | 4.863e-01 | 1.000000 | 1.000000 | - | - |
| 0.246575 | 13 | 100.00% | 0.000e+00 | 6.662e-10 | 0.000 | 4.786e-02 | 4.775e-01 | 1.000000 | 1.000000 | - | - |
| 0.493151 | 13 | 100.00% | 0.000e+00 | 4.237e-08 | 0.000 | 5.834e-02 | 5.088e-01 | 1.000000 | 1.000000 | - | - |
| 0.739726 | 13 | 100.00% | 0.000e+00 | 3.218e-09 | 0.000 | 6.571e-02 | 5.387e-01 | 1.000000 | 1.000000 | - | - |
| 1.000000 | 13 | 100.00% | 0.000e+00 | 5.304e-09 | 0.000 | 7.263e-02 | 5.647e-01 | 1.000000 | 1.000000 | - | - |
| 2.000000 | 13 | 100.00% | 0.000e+00 | 2.043e-07 | 0.000 | 8.482e-02 | 6.290e-01 | 1.000000 | 1.000000 | - | - |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/sabr/sabr_catalog_06_steep_upward_term_structure__default/sabr_catalog_06_steep_upward_term_structure__default_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/sabr/sabr_catalog_06_steep_upward_term_structure__default/sabr_catalog_06_steep_upward_term_structure__default_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/sabr/sabr_catalog_06_steep_upward_term_structure__default/sabr_catalog_06_steep_upward_term_structure__default_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/sabr/sabr_catalog_06_steep_upward_term_structure__default/sabr_catalog_06_steep_upward_term_structure__default_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/sabr/sabr_catalog_06_steep_upward_term_structure__default/sabr_catalog_06_steep_upward_term_structure__default_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/sabr/sabr_catalog_06_steep_upward_term_structure__default/sabr_catalog_06_steep_upward_term_structure__default_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/sabr/sabr_catalog_06_steep_upward_term_structure__default`

