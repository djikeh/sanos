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
- Objective value: `1.424276e+10`
- Inside bid/ask ratio: `29.29%`
- Bid/ask violation (max / mean): `2.478e-03` / `1.439e-04`
- MAE(mid): `1.439e-04`
- RMSE(mid): `3.575e-04`
- MAE(residual/half-spread): `287732455.026`
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

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 15 | 6.67% | 8.224e-05 | 1.671e-05 | 33423829.143 | 7.535e-02 | 3.648e+00 | 1.000000 | 1.000000 | - | - |
| 0.027397 | 15 | 6.67% | 1.400e-04 | 1.850e-05 | 37004820.789 | 7.204e-02 | 3.440e+00 | 1.000000 | 1.000000 | - | - |
| 0.038356 | 15 | 40.00% | 2.204e-04 | 3.236e-05 | 64727705.000 | 7.550e-02 | 4.788e+00 | 1.000000 | 1.000000 | - | - |
| 0.246575 | 15 | 20.00% | 5.441e-04 | 1.864e-04 | 372749334.937 | 4.227e-02 | 1.273e+00 | 1.000000 | 1.000000 | - | - |
| 0.249315 | 13 | 46.15% | 4.609e-04 | 7.721e-05 | 154424931.797 | 3.894e-02 | 9.181e-01 | 1.000000 | 1.000000 | - | - |
| 1.000000 | 13 | 61.54% | 8.584e-04 | 2.002e-04 | 400382714.496 | 3.516e-02 | 8.444e-01 | 1.000000 | 1.000000 | - | - |
| 2.000000 | 13 | 30.77% | 2.478e-03 | 5.252e-04 | 1050340638.285 | 4.064e-02 | 7.996e-01 | 1.000000 | 1.000000 | - | - |

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

