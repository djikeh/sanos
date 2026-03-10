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
- Objective value: `4.579900e-03`
- Inside bid/ask ratio: `99.39%`
- Bid/ask violation (max / mean): `2.305e-10` / `1.397e-12`
- MAE(mid): `6.916e-08`
- RMSE(mid): `4.455e-07`
- MAE(residual/half-spread): `0.132`
- IV total variation: `5.326e-02`
- IV max second diff: `3.323e-01`
- MAE(iv): `5.325e-07`
- RMSE(iv): `3.279e-06`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=6.916e-08, RMSE=4.455e-07. Erreur en volatilite implicite: MAE=5.325e-07, RMSE=3.279e-06. Rugosite IV (global): TV=5.326e-02, max seconde diff=3.323e-01.

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 33 | 96.97% | 2.305e-10 | 3.531e-09 | 0.661 | 1.026e-02 | 3.323e-01 | 1.000000 | 1.000000 | - | - |
| 0.082192 | 33 | 100.00% | 0.000e+00 | 9.535e-11 | 0.000 | 1.162e-02 | 2.796e-01 | 1.000000 | 1.000000 | - | - |
| 0.246575 | 33 | 100.00% | 0.000e+00 | 3.460e-08 | 0.000 | 1.067e-02 | 2.412e-01 | 1.000000 | 1.000000 | - | - |
| 0.493151 | 33 | 100.00% | 0.000e+00 | 2.877e-07 | 0.000 | 1.043e-02 | 2.333e-01 | 1.000000 | 1.000000 | - | - |
| 1.000000 | 33 | 100.00% | 0.000e+00 | 1.987e-08 | 0.000 | 1.027e-02 | 2.205e-01 | 1.000000 | 1.000000 | - | - |

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

