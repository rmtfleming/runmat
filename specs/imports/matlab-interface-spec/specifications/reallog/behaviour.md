# `reallog` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/reallog_observations.json`](observations/reallog_observations.json)

## Summary

Return the natural logarithm of non-negative real input, erroring instead of producing complex results.

## Confirmed invocation forms

- `y = reallog(x)` — Natural log of x. *(e.g. case `one`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Values equal the natural log: reallog(1) = 0, reallog(exp(2)) = 2, [1 10 100] gave [0 log(10) log(100)] (within 1e-12). *(cases: `one`, `e`, `vector`)*
2. **Confirmed** — reallog(0) = -Inf, reallog(Inf) = Inf, reallog(NaN) = NaN. *(cases: `zero`, `inf`, `nan`)*
3. **Confirmed** — Matrix shape is kept; single input gives single; [] gives []. *(cases: `matrix`, `single`, `empty`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| x contains a negative value. | `MATLAB:reallog:complexResult` | `err-negative` |
| x is complex. | `MATLAB:validators:mustBeReal` | `err-complex` |
| x is of an integer class. | `MATLAB:math:mustBeFloat` | `err-int` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `one` | `double` | `[1, 1]` | 0 |
| `e` | `double` | `[1, 1]` | 2 |
| `vector` | `double` | `[1, 3]` | [0 2.3025850929940459 4.6051701859880918] |
| `zero` | `double` | `[1, 1]` | -Inf |
| `inf` | `double` | `[1, 1]` | Inf |
| `nan` | `double` | `[1, 1]` | NaN |
| `matrix` | `double` | `[2, 2]` | [0 0.69314718055994529;1.0986122886681098 1.3862943611198906] |
| `single` | `single` | `[1, 1]` | 1.3862943649291992 |
| `empty` | `double` | `[0, 0]` | [] |
| `err-negative` | — | — | error `MATLAB:reallog:complexResult` |
| `err-complex` | — | — | error `MATLAB:validators:mustBeReal` |
| `err-int` | — | — | error `MATLAB:math:mustBeFloat` |

## Unresolved questions

- `reallog.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
