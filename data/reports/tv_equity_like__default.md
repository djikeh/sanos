# Calibration Report - `tv_equity_like__default`

## Inputs
- Config: `C:\Users\djike\Dev\Repos\paper\sanos\data\configs\default.json`
- Snapshot: `C:\Users\djike\Dev\Repos\paper\sanos\data\snapshots\tv_equity_like.json`

## Outputs
- Surface JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\tv_equity_like__default\surface.json`
- Diagnostics JSON: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\tv_equity_like__default\diagnostics.json`
- Surface folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\surfaces\tv_equity_like__default`
- Images folder: `C:\Users\djike\Dev\Repos\paper\sanos\data\images\tv_equity_like__default`

## Global Metrics
- Number of quotes: `155`
- Objective value: `2.728122e-03`
- Inside bid/ask ratio: `100.00%`
- Bid/ask violation (max / mean): `0.000e+00` / `0.000e+00`
- MAE(mid): `1.944e-08`
- RMSE(mid): `3.399e-08`
- MAE(residual/half-spread): `0.000`
- IV total variation: `4.862e-01`
- IV max second diff: `1.676e+00`
- MAE(iv): `3.023e-07`
- RMSE(iv): `1.071e-06`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises sont globalement contenus par rapport au demi-spread. Erreur en prix: MAE=1.944e-08, RMSE=3.399e-08. Erreur en volatilite implicite: MAE=3.023e-07, RMSE=1.071e-06. Rugosite IV (global): TV=4.862e-01, max seconde diff=1.676e+00.

## Oscillation Analysis (Before/After)
- Baseline IV smoothness (no init): TV=`4.860e-01`, max second diff=`1.679e+00`
- Current IV smoothness (linear-density init): TV=`4.862e-01`, max second diff=`1.676e+00`
- Delta (current - baseline): TV=`1.549e-04`, max second diff=`-3.617e-03`
Conclusion: the IV oscillation level decreases after enabling linear-density initialization.

Most improved maturities (delta max second diff):
- T=0.019178, delta=-3.617e-03, baseline strike=0.7200, current strike=0.7200
- T=0.493151, delta=-3.293e-07, baseline strike=0.9000, current strike=0.9000
- T=0.246575, delta=-2.672e-08, baseline strike=0.9000, current strike=0.9000
- T=0.082192, delta=2.469e-05, baseline strike=0.7200, current strike=0.7200

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 31 | 100.00% | 0.000e+00 | 8.104e-11 | 0.000 | 1.369e-01 | 1.676e+00 | 1.000000 | 1.000000 | - | - |
| 0.082192 | 31 | 100.00% | 0.000e+00 | 2.844e-08 | 0.000 | 1.068e-01 | 1.316e+00 | 1.000000 | 1.000000 | - | - |
| 0.246575 | 31 | 100.00% | 0.000e+00 | 7.345e-09 | 0.000 | 8.968e-02 | 1.087e+00 | 1.000000 | 1.000000 | - | - |
| 0.493151 | 31 | 100.00% | 0.000e+00 | 1.925e-08 | 0.000 | 8.054e-02 | 9.760e-01 | 1.000000 | 1.000000 | - | - |
| 1.000000 | 31 | 100.00% | 0.000e+00 | 4.208e-08 | 0.000 | 7.228e-02 | 8.761e-01 | 1.000000 | 1.000000 | - | - |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Price smiles: `../images/tv_equity_like__default/tv_equity_like__default_smiles_fit.png`
  Interpretation: verifier si la courbe `Model` reste dans la bande `Bid/Ask` sur tous les strikes.
- IV smiles: `../images/tv_equity_like__default/tv_equity_like__default_smiles_iv_fit.png`
  Interpretation: controler la coherence de skew/smile en volatilite implicite, pas seulement en prix.
- Residual heatmap: `../images/tv_equity_like__default/tv_equity_like__default_residual_heatmap.png`
  Interpretation: rechercher des zones systematiques de sous/sur-pricing selon maturite et strike.
- Quality summary: `../images/tv_equity_like__default/tv_equity_like__default_quality_summary.png`
  Interpretation: vue compacte des erreurs et de la distribution des residus normalises.
- Surface heatmap: `../images/tv_equity_like__default/tv_equity_like__default_surface_heatmap.png`
  Interpretation: continuite globale de la surface en maturite/strike.
- Density comparison: `../images/tv_equity_like__default/tv_equity_like__default_density_comparison.png`
  Interpretation: comparer la densite implicite `d2C/dK2` a la densite discrete SANOS (pics `q`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../surfaces/tv_equity_like__default`

