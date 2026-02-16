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
- Objective value: `5.091530e-04`
- Inside bid/ask ratio: `100.00%`
- Bid/ask violation (max / mean): `1.110e-16` / `1.256e-18`
- MAE(mid): `6.873e-04`
- RMSE(mid): `1.050e-03`
- MAE(residual/half-spread): `0.622`
- IV total variation: `4.578e-01`
- IV max second diff: `2.164e+01`
- MAE(iv): `4.359e-03`
- RMSE(iv): `4.918e-03`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises restent raisonnables mais sont proches de la largeur des spreads. Erreur en prix: MAE=6.873e-04, RMSE=1.050e-03. Erreur en volatilite implicite: MAE=4.359e-03, RMSE=4.918e-03. Rugosite IV (global): TV=4.578e-01, max seconde diff=2.164e+01.

## Oscillation Analysis (Before/After)
- Baseline IV smoothness (no init): TV=`5.585e-01`, max second diff=`3.148e+01`
- Current IV smoothness (linear-density init): TV=`4.578e-01`, max second diff=`2.164e+01`
- Delta (current - baseline): TV=`-1.008e-01`, max second diff=`-9.842e+00`
Conclusion: the IV oscillation level decreases after enabling linear-density initialization.

Most improved maturities (delta max second diff):
- T=0.019178, delta=-9.842e+00, baseline strike=0.7200, current strike=0.7400
- T=0.246575, delta=-4.646e+00, baseline strike=0.7400, current strike=0.7400
- T=0.082192, delta=-3.812e+00, baseline strike=0.7800, current strike=0.8000
- T=0.493151, delta=-2.840e+00, baseline strike=0.7200, current strike=0.8200

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 31 | 100.00% | 2.776e-17 | 6.554e-05 | 0.660 | 1.584e-01 | 2.164e+01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.082192 | 31 | 100.00% | 0.000e+00 | 2.275e-04 | 0.645 | 1.171e-01 | 5.239e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.246575 | 31 | 100.00% | 5.551e-17 | 5.698e-04 | 0.652 | 7.203e-02 | 2.473e+00 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 0.493151 | 31 | 100.00% | 0.000e+00 | 9.099e-04 | 0.552 | 6.077e-02 | 8.291e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |
| 1.000000 | 31 | 100.00% | 1.110e-16 | 1.664e-03 | 0.598 | 4.950e-02 | 6.323e-01 | 1.000000 | 1.000000 | no | 0.000e+00 |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Plots skipped (`--no-plot`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `..\surfaces\tv_equity_like__default`

