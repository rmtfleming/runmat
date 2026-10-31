# `condest` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/condest_observations.json`](observations/condest_observations.json)

## Summary

Estimate the 1-norm condition number of a square matrix, and optionally a vector v (see two-out).

## Confirmed invocation forms

- `c = condest(A)` — 1-norm condition estimate (COBRA form). *(e.g. case `identity`)*
- `[c, v] = condest(A)` — Also a vector v. *(e.g. case `two-out`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — speye(4) gave 1; sparse diag([1 10 100]) gave 100; full [2 0; 0 8] gave 4; sparse(5) gave 1. *(cases: `identity`, `diag`, `dense`, `scalar`)*
2. **Confirmed** — For a 4x4 tridiagonal matrix condest equalled cond(full(A), 1) (2.7272727272727271); for a 3x3 unsymmetric matrix they agreed to within 1e-15 (3.4000000000000004 vs 3.3999999999999995). *(cases: `vs-cond`, `vs-cond-unsym`)*
3. **Confirmed** — A singular or all-zero sparse matrix gives Inf. *(cases: `singular`, `zero`)*
4. **Confirmed** — A 0x0 input gives 0. *(cases: `empty`)*
5. **Confirmed** — For sparse diag([1 10 100]), [c, v] gave c = 100 and a 3x1 v with norm(v,1) = 1 and norm(A*v,1) = 1. *(cases: `two-out`)*

## Edge cases (observed)

- *(non-normative)* Inference (not observed): condest is an estimate, so only the order of magnitude, not exact agreement with cond(A, 1), should be required beyond the observed cases. *(cases: `vs-cond-unsym`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A is not square. | `MATLAB:condest:NonSquareMatrix` | `err-nonsquare` |
| A is a square char matrix. | `MATLAB:lu:inputType` | `err-char` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `identity` | `double` | `[1, 1]` | 1 |
| `diag` | `double` | `[1, 1]` | 100 |
| `vs-cond` | `struct` | `[1, 1]` | condest=2.7272727272727271; cond1=2.7272727272727271 |
| `vs-cond-unsym` | `struct` | `[1, 1]` | condest=3.4000000000000004; cond1=3.3999999999999995 |
| `dense` | `double` | `[1, 1]` | 4 |
| `singular` | `double` | `[1, 1]` | Inf |
| `zero` | `double` | `[1, 1]` | Inf |
| `two-out` | `struct` | `[1, 1]` | c=100; size_v=[3 1]; norm1_v=1; norm1_Av=1 |
| `scalar` | `double` | `[1, 1]` | 1 |
| `empty` | `double` | `[1, 1]` | 0 |
| `err-nonsquare` | — | — | error `MATLAB:condest:NonSquareMatrix` |
| `err-char` | — | — | error `MATLAB:lu:inputType` |

## Unresolved questions

- `condest.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `condest.q-error-ids` — The char-input identifier names another function (lu); whether an implementation must reproduce it is a human-review question (constitution 8.2).
- `condest.q-larger` — Only matrices up to 4x4 were exercised; agreement with cond(A, 1) on larger matrices was not observed.
- `condest.q-exactness` — Whether the exact observed values in vs-cond should be required, or only agreement with cond(A, 1) to a tolerance, is a human-review question (constitution 8.2).
