# Calibration Report - `tv_catalog_04_strong_wings__default`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\default.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_04_strong_wings.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_04_strong_wings__default\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_04_strong_wings__default\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_04_strong_wings__default`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_04_strong_wings__default`

## Global Metrics
- Number of quotes: `287`
- Objective value: `4.320790e+01`
- Inside bid/ask ratio: `89.90%`
- Bid/ask violation (max / mean): `1.671e-02` / `7.173e-04`
- MAE(mid): `1.633e-03`
- RMSE(mid): `4.659e-03`
- MAE(residual/half-spread): `0.313`
- IV total variation: `2.512e+00`
- IV max second diff: `1.153e+01`
- MAE(iv): `4.376e-03`
- RMSE(iv): `1.158e-02`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
Le taux inside bid/ask est faible: la calibration merite un reglage des hyperparametres. Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=1.633e-03, RMSE=4.659e-03. Erreur en volatilite implicite: MAE=4.376e-03, RMSE=1.158e-02. Rugosite IV (global): TV=2.512e+00, max seconde diff=1.153e+01.

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 41 | 100.00% | 0.000e+00 | 1.780e-06 | 0.091 | 4.792e-01 | 1.153e+01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.027397 | 41 | 100.00% | 0.000e+00 | 7.017e-06 | 0.087 | 4.300e-01 | 6.162e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.038356 | 41 | 100.00% | 0.000e+00 | 5.813e-06 | 0.060 | 4.266e-01 | 6.670e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.246575 | 41 | 100.00% | 0.000e+00 | 6.644e-05 | 0.059 | 3.653e-01 | 3.126e+00 | 1.000000 | 1.000000 | yes | 4.022e-02 |
| 0.249315 | 41 | 100.00% | 0.000e+00 | 7.377e-05 | 0.065 | 3.670e-01 | 3.872e+00 | 1.000000 | 1.000000 | yes | 3.113e-02 |
| 1.000000 | 41 | 97.56% | 1.529e-03 | 6.822e-04 | 0.207 | 3.052e-01 | 2.699e+00 | 1.000000 | 1.000000 | yes | 3.557e-01 |
| 2.000000 | 41 | 31.71% | 1.671e-02 | 1.059e-02 | 1.625 | 1.391e-01 | 1.625e+00 | 1.000000 | 1.000000 | yes | 1.022e+00 |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_04_strong_wings__default/tv_catalog_04_strong_wings__default_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_04_strong_wings__default/tv_catalog_04_strong_wings__default_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_04_strong_wings__default/tv_catalog_04_strong_wings__default_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_04_strong_wings__default/tv_catalog_04_strong_wings__default_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_04_strong_wings__default/tv_catalog_04_strong_wings__default_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_04_strong_wings__default/tv_catalog_04_strong_wings__default_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_04_strong_wings__default`

