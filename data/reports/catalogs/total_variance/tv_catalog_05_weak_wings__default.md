# Calibration Report - `tv_catalog_05_weak_wings__default`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\default.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_05_weak_wings.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_05_weak_wings__default\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_05_weak_wings__default\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_05_weak_wings__default`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_05_weak_wings__default`

## Global Metrics
- Number of quotes: `165`
- Objective value: `6.644233e-03`
- Inside bid/ask ratio: `100.00%`
- Bid/ask violation (max / mean): `0.000e+00` / `0.000e+00`
- MAE(mid): `7.757e-08`
- RMSE(mid): `4.599e-07`
- MAE(residual/half-spread): `0.000`
- IV total variation: `5.782e-02`
- IV max second diff: `3.662e-01`
- MAE(iv): `3.792e-07`
- RMSE(iv): `2.303e-06`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=7.757e-08, RMSE=4.599e-07. Erreur en volatilite implicite: MAE=3.792e-07, RMSE=2.303e-06. Rugosite IV (global): TV=5.782e-02, max seconde diff=3.662e-01.

## Oscillation Analysis (Before/After)
- Baseline IV smoothness (no init): TV=`5.326e-02`, max second diff=`3.323e-01`
- Current IV smoothness (linear-density init): TV=`5.782e-02`, max second diff=`3.662e-01`
- Delta (current - baseline): TV=`4.567e-03`, max second diff=`3.386e-02`
Conclusion: oscillation did not improve globally; inspect per-maturity rows below.

Most improved maturities (delta max second diff):
- T=1.000000, delta=-1.571e-04, baseline strike=0.8514, current strike=0.8514
- T=0.246575, delta=-2.137e-08, baseline strike=0.8514, current strike=0.8514
- T=0.082192, delta=2.902e-07, baseline strike=0.8514, current strike=0.8514
- T=0.493151, delta=3.034e-04, baseline strike=0.8514, current strike=0.8514

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 33 | 100.00% | 0.000e+00 | 3.856e-09 | 0.000 | 1.483e-02 | 3.662e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.082192 | 33 | 100.00% | 0.000e+00 | 4.828e-10 | 0.000 | 1.162e-02 | 2.796e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.246575 | 33 | 100.00% | 0.000e+00 | 4.879e-08 | 0.000 | 1.067e-02 | 2.412e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.493151 | 33 | 100.00% | 0.000e+00 | 2.982e-07 | 0.000 | 1.043e-02 | 2.336e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 1.000000 | 33 | 100.00% | 0.000e+00 | 3.646e-08 | 0.000 | 1.027e-02 | 2.203e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_05_weak_wings__default/tv_catalog_05_weak_wings__default_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_05_weak_wings__default/tv_catalog_05_weak_wings__default_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_05_weak_wings__default/tv_catalog_05_weak_wings__default_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_05_weak_wings__default/tv_catalog_05_weak_wings__default_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_05_weak_wings__default/tv_catalog_05_weak_wings__default_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_05_weak_wings__default/tv_catalog_05_weak_wings__default_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_05_weak_wings__default`

