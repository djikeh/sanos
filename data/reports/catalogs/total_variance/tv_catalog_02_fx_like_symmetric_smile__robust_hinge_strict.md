# Calibration Report - `tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\robust_hinge_strict.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\catalogs\total_variance\tv_catalog_02_fx_like_symmetric_smile.snapshot.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\catalogs\total_variance\tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\catalogs\total_variance\tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict`

## Global Metrics
- Number of quotes: `333`
- Objective value: `2.333295e+00`
- Inside bid/ask ratio: `99.70%`
- Bid/ask violation (max / mean): `1.550e-11` / `4.656e-14`
- MAE(mid): `9.302e-05`
- RMSE(mid): `1.578e-04`
- MAE(residual/half-spread): `0.158`
- IV total variation: `4.088e-01`
- IV max second diff: `9.771e+00`
- MAE(iv): `4.467e-04`
- RMSE(iv): `1.491e-03`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=9.302e-05, RMSE=1.578e-04. Erreur en volatilite implicite: MAE=4.467e-04, RMSE=1.491e-03. Rugosite IV (global): TV=4.088e-01, max seconde diff=9.771e+00.

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 37 | 97.30% | 1.550e-11 | 1.218e-05 | 0.848 | 7.675e-02 | 9.771e+00 | 1.000000 | 1.000000 | yes | 1.609e-01 |
| 0.038356 | 37 | 100.00% | 0.000e+00 | 2.302e-05 | 0.069 | 4.634e-02 | 2.072e+00 | 1.000000 | 1.000000 | yes | 2.645e-01 |
| 0.082192 | 37 | 100.00% | 0.000e+00 | 3.364e-05 | 0.067 | 4.214e-02 | 1.486e+00 | 1.000000 | 1.000000 | yes | 3.342e-01 |
| 0.164384 | 37 | 100.00% | 0.000e+00 | 5.609e-05 | 0.075 | 4.081e-02 | 1.416e+00 | 1.000000 | 1.000000 | yes | 7.150e-01 |
| 0.246575 | 37 | 100.00% | 0.000e+00 | 8.238e-05 | 0.087 | 4.150e-02 | 1.235e+00 | 1.000000 | 1.000000 | yes | 7.740e-01 |
| 0.493151 | 37 | 100.00% | 0.000e+00 | 1.173e-04 | 0.079 | 4.146e-02 | 9.889e-01 | 1.000000 | 1.000000 | yes | 1.466e+00 |
| 0.739726 | 37 | 100.00% | 0.000e+00 | 1.363e-04 | 0.070 | 4.024e-02 | 8.585e-01 | 1.000000 | 1.000000 | yes | 4.539e+00 |
| 1.000000 | 37 | 100.00% | 0.000e+00 | 1.821e-04 | 0.077 | 4.032e-02 | 9.924e-01 | 1.000000 | 1.000000 | yes | 2.048e+00 |
| 2.000000 | 37 | 100.00% | 0.000e+00 | 1.942e-04 | 0.051 | 3.928e-02 | 8.886e-01 | 1.000000 | 1.000000 | yes | 1.249e+00 |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../../../images/catalogs/total_variance/tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict/tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../../../images/catalogs/total_variance/tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict/tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../../../images/catalogs/total_variance/tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict/tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../../../images/catalogs/total_variance/tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict/tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../../../images/catalogs/total_variance/tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict/tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../../../images/catalogs/total_variance/tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict/tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../../../surfaces/catalogs/total_variance/tv_catalog_02_fx_like_symmetric_smile__robust_hinge_strict`

