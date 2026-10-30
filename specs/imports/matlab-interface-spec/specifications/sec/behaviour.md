# `sec` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/sec_observations.json`](observations/sec_observations.json)

## Summary

Elementwise secant of floating-point input.

## Confirmed invocation forms

- `Y = sec(X)` — Secant of each element. *(e.g. case `zero`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — sec(0) = 1, sec(pi) = -1, sec(pi/3) = 1.9999999999999996, sec(-pi/4) = 1.4142135623730949 (within 1e-12). *(cases: `zero`, `pi`, `pi3`, `negative`)*
2. **Confirmed** — sec(pi/2) is finite and large (16331239353195370 observed). *(cases: `pole`)*
3. **Confirmed** — Output has the size of the input, including empty. *(cases: `matrix`, `empty`)*
4. **Confirmed** — sec(NaN) and sec(Inf) are NaN. *(cases: `nan`, `inf`)*
5. **Confirmed** — Complex input gives a complex result: sec(1+1i) = 0.49833703055518686+0.59108384172104511i. *(cases: `complex`)*
6. **Confirmed** — Single input gives a single result. *(cases: `single`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| X is logical, int8 (the one integer class tried) or char. | `MATLAB:math:mustBeFloat` | `err-logical`, `err-int`, `err-char` |
| No input is given. | `MATLAB:minrhs` | `err-none` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `zero` | `double` | `[1, 1]` | 1 |
| `pi3` | `double` | `[1, 1]` | 1.9999999999999996 |
| `pi` | `double` | `[1, 1]` | -1 |
| `negative` | `double` | `[1, 1]` | 1.4142135623730949 |
| `pole` | `double` | `[1, 1]` | 16331239353195370 |
| `matrix` | `double` | `[2, 2]` | [1 -1;1.9999999999999996 1.9999999999999996] |
| `nan` | `double` | `[1, 1]` | NaN |
| `inf` | `double` | `[1, 1]` | NaN |
| `complex` | `double` | `[1, 1]` | 0.49833703055518686+0.59108384172104511i |
| `single` | `single` | `[1, 1]` | 1.8508157730102539 |
| `empty` | `double` | `[0, 0]` | [] |
| `err-logical` | — | — | error `MATLAB:math:mustBeFloat` |
| `err-int` | — | — | error `MATLAB:math:mustBeFloat` |
| `err-char` | — | — | error `MATLAB:math:mustBeFloat` |
| `err-none` | — | — | error `MATLAB:minrhs` |

## Unresolved questions

- `sec.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
