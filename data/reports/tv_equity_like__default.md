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
- Objective value: `-0.000000e+00`
- Inside bid/ask ratio: `100.00%`
- Bid/ask violation (max / mean): `1.110e-16` / `2.865e-18`
- MAE(mid): `7.360e-04`
- RMSE(mid): `1.086e-03`
- MAE(residual/half-spread): `0.659`
- IV total variation: `5.585e-01`
- IV max second diff: `3.148e+01`
- MAE(iv): `4.691e-03`
- RMSE(iv): `5.223e-03`

## No-Arbitrage Diagnostics
- Strike monotonicity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Strike convexity violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`
- Calendar violation (max / mean / count): `0.000e+00` / `0.000e+00` / `0`

### Interpretation automatique
La calibration est tres coherente avec le bid/ask (quasi toutes les quotes sont dans le spread). Les residus normalises restent raisonnables mais sont proches de la largeur des spreads. Erreur en prix: MAE=7.360e-04, RMSE=1.086e-03. Erreur en volatilite implicite: MAE=4.691e-03, RMSE=5.223e-03. Rugosite IV (global): TV=5.585e-01, max seconde diff=3.148e+01.

## Per Maturity
| T | n_quotes | inside | bid/ask max | MAE(mid) | MAE(norm) | TV(iv) | max d2(iv) | density mass | density mean | proj? | proj L1 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.019178 | 31 | 100.00% | 4.510e-17 | 5.945e-05 | 0.654 | 1.525e-01 | 3.148e+01 | 1.000000 | 1.000000 | - | - |
| 0.082192 | 31 | 100.00% | 1.110e-16 | 2.307e-04 | 0.678 | 1.308e-01 | 9.050e+00 | 1.000000 | 1.000000 | - | - |
| 0.246575 | 31 | 100.00% | 1.006e-16 | 5.712e-04 | 0.639 | 1.141e-01 | 7.119e+00 | 1.000000 | 1.000000 | - | - |
| 0.493151 | 31 | 100.00% | 4.163e-17 | 1.002e-03 | 0.649 | 9.276e-02 | 3.669e+00 | 1.000000 | 1.000000 | - | - |
| 1.000000 | 31 | 100.00% | 0.000e+00 | 1.817e-03 | 0.673 | 6.845e-02 | 1.052e+00 | 1.000000 | 1.000000 | - | - |

Linear density discretization used:
- Node-based second-difference on strike grid: `p0=1+d0`, `pi=di-d(i-1)` for interior nodes, `pN=-d(N-1)`, with `di=(C(i+1)-Ci)/(K(i+1)-Ki)`.
- If raw `p` is infeasible, projection solves `min ||p*-p||_1` under `p*>=0`, `sum(p*)=1`, `sum(p* K)=1`.

## Figures And Interpretation
- Plots skipped (`--no-plot`).

## Reconstructability
`surface.json` inclut un bloc `reconstruction` permettant de rebatir `SanosSurface` sans la config ni le snapshot d'origine.
- Surface JSON folder: `../surfaces/tv_equity_like__default`

