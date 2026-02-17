# Calibration Report - `tv_catalog_09_twist__default`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\default.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_09_twist.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_09_twist__default\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_09_twist__default\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_09_twist__default`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_09_twist__default`

## Global Metrics
- Number of quotes: `315`
- Objective value: `7.185961e+00`
- Inside bid/ask ratio: `99.37%`
- Bid/ask violation (max / mean): `3.703e-09` / `2.347e-11`
- MAE(mid): `8.227e-05`
- RMSE(mid): `1.833e-04`
- MAE(residual/half-spread): `0.591`
- IV total variation: `4.722e-01`
- IV max second diff: `4.431e+00`
- MAE(iv): `3.316e-04`
- RMSE(iv): `5.817e-04`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises restent raisonnables mais sont proches de la largeur des spreads. Erreur en prix: MAE=8.227e-05, RMSE=1.833e-04. Erreur en volatilite implicite: MAE=3.316e-04, RMSE=5.817e-04. Rugosite IV (global): TV=4.722e-01, max seconde diff=4.431e+00.

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 35 | 97.14% | 3.703e-09 | 1.244e-05 | 4.624 | 1.115e-01 | 4.431e+00 | 1.000000 | 1.000000 | yes | 1.511e-01 |
| 0.038356 | 35 | 97.14% | 3.690e-09 | 1.925e-05 | 0.376 | 5.843e-02 | 2.202e+00 | 1.000000 | 1.000000 | yes | 2.919e-01 |
| 0.082192 | 35 | 100.00% | 0.000e+00 | 3.041e-05 | 0.050 | 4.459e-02 | 1.434e+00 | 1.000000 | 1.000000 | yes | 3.381e-01 |
| 0.164384 | 35 | 100.00% | 0.000e+00 | 3.864e-05 | 0.041 | 4.015e-02 | 1.043e+00 | 1.000000 | 1.000000 | yes | 5.263e-01 |
| 0.246575 | 35 | 100.00% | 0.000e+00 | 4.248e-05 | 0.037 | 3.700e-02 | 9.066e-01 | 1.000000 | 1.000000 | yes | 2.212e-01 |
| 0.493151 | 35 | 100.00% | 0.000e+00 | 8.223e-05 | 0.044 | 4.251e-02 | 8.849e-01 | 1.000000 | 1.000000 | yes | 9.731e-01 |
| 0.739726 | 35 | 100.00% | 0.000e+00 | 1.211e-04 | 0.050 | 4.468e-02 | 1.017e+00 | 1.000000 | 1.000000 | yes | 1.009e+00 |
| 1.000000 | 35 | 100.00% | 0.000e+00 | 1.392e-04 | 0.047 | 4.697e-02 | 6.092e-01 | 1.000000 | 1.000000 | yes | 1.829e+00 |
| 2.000000 | 35 | 100.00% | 0.000e+00 | 2.547e-04 | 0.052 | 4.642e-02 | 5.791e-01 | 1.000000 | 1.000000 | yes | 1.134e+00 |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_09_twist__default/tv_catalog_09_twist__default_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_09_twist__default/tv_catalog_09_twist__default_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_09_twist__default/tv_catalog_09_twist__default_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_09_twist__default/tv_catalog_09_twist__default_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_09_twist__default/tv_catalog_09_twist__default_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_09_twist__default/tv_catalog_09_twist__default_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_09_twist__default`

