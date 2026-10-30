# `nthroot` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/nthroot_observations.json`](observations/nthroot_observations.json)

## Summary

Return the real n-th root of real input, elementwise, including real odd roots of negative values.

## Confirmed invocation forms

- `y = nthroot(x, n)` — Real n-th root of x. *(e.g. case `cube-neg`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is real with the class of x (double; single for single x). *(cases: `cube-neg`, `single`)*
2. **Confirmed** — Odd roots of negative values are real and negative: nthroot(-8, 3) = -2; [1 8 27 -64] with n = 3 gave [1 2 3 -4]. *(cases: `cube-neg`, `vector`)*
3. **Confirmed** — An even root of a positive value is the positive real root: nthroot(16, 4) gave 2 (within 1e-12). *(cases: `fourth`)*
4. **Confirmed** — n may be an array matching x: [4 27 16] with [2 3 4] gave [2 3 2]. *(cases: `vector-n`)*
5. **Confirmed** — 0, Inf, -Inf and NaN with n = 3 gave 0, Inf, -Inf, NaN. *(cases: `specials`)*
6. **Confirmed** — n = 1 returns x; n = -3 gave 8^(-1/3) = 0.5; a non-integer n (2.5) with positive x gave 8^(1/2.5). *(cases: `n-one`, `n-negative`, `noninteger-n`)*
7. **Confirmed** — nthroot(1000, 3) == 10 was true. *(cases: `exactness`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| x is negative and n is not an odd integer (n = 4, or n = 2.5). | `MATLAB:nthroot:NegXNotOddIntegerN` | `err-even-negative`, `err-noninteger-n-neg` |
| x is complex. | `MATLAB:nthroot:ComplexInput` | `err-complex` |
| x and n have incompatible sizes. | `MATLAB:sizeDimensionsMustMatch` | `err-size` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `cube-neg` | `double` | `[1, 1]` | -2 |
| `fourth` | `double` | `[1, 1]` | 2 |
| `vector` | `double` | `[1, 4]` | [1 2 3 -4] |
| `vector-n` | `double` | `[1, 3]` | [2 3 2] |
| `specials` | `double` | `[1, 4]` | [0 Inf -Inf NaN] |
| `n-one` | `double` | `[1, 1]` | -5 |
| `n-negative` | `double` | `[1, 1]` | 0.5 |
| `single` | `single` | `[1, 1]` | 3 |
| `exactness` | `logical` | `[1, 1]` | true |
| `err-even-negative` | — | — | error `MATLAB:nthroot:NegXNotOddIntegerN` |
| `noninteger-n` | `double` | `[1, 1]` | 2.2973967099940702 |
| `err-noninteger-n-neg` | — | — | error `MATLAB:nthroot:NegXNotOddIntegerN` |
| `err-complex` | — | — | error `MATLAB:nthroot:ComplexInput` |
| `err-size` | — | — | error `MATLAB:sizeDimensionsMustMatch` |

## Unresolved questions

- `nthroot.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
