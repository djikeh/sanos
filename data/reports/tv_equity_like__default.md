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
- Inside bid/ask ratio: `100.00%`
- MAE(mid): `7.360e-04`
- RMSE(mid): `1.086e-03`
- MAE(residual/half-spread): `0.659`
- MAE(iv): `4.691e-03`
- RMSE(iv): `5.223e-03`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises restent raisonnables mais sont proches de la largeur des spreads. Erreur en prix: MAE=7.360e-04, RMSE=1.086e-03. Erreur en volatilite implicite: MAE=4.691e-03, RMSE=5.223e-03.

## Per Maturity
| T | n_quotes | inside | MAE(mid) | RMSE(mid) | MAE(norm) | MAE(iv) |
|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 31 | 100.00% | 5.945e-05 | 9.792e-05 | 0.654 | 4.022e-03 |
| 0.082192 | 31 | 100.00% | 2.307e-04 | 2.951e-04 | 0.678 | 4.382e-03 |
| 0.246575 | 31 | 100.00% | 5.712e-04 | 6.718e-04 | 0.639 | 4.492e-03 |
| 0.493151 | 31 | 100.00% | 1.002e-03 | 1.135e-03 | 0.649 | 4.935e-03 |
| 1.000000 | 31 | 100.00% | 1.817e-03 | 2.016e-03 | 0.673 | 5.625e-03 |

## Figures And Interpretation
- Plots skipped (`--no-plot`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `..\surfaces\tv_equity_like__default`

