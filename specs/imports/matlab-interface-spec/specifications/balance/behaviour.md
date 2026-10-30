# `balance` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/balance_observations.json`](observations/balance_observations.json)

## Summary

Balance a square matrix by a diagonal (power-of-two) similarity scaling, possibly with a permutation, to improve eigenvalue accuracy.

## Confirmed invocation forms

- `B = balance(A)` — Balanced matrix. *(e.g. case `single-output`)*
- `[T, B] = balance(A)` — Transformation T with B = T\A*T. *(e.g. case `two-outputs`)*
- `[S, P, B] = balance(A)` — Scaling vector S and permutation vector P. *(e.g. case `three-outputs`)*
- `___ = balance(A, 'noperm')` — Scale without permuting. *(e.g. case `noperm`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output keeps the input class (double for double; single for single). *(cases: `single-output`, `single-class`)*
2. **Confirmed** — [T, B] = balance(A) satisfies T\A*T = B within 1e-12 (residual 0 observed for both matrices tried) and preserves the eigenvalues (difference within 1e-10). *(cases: `two-outputs`, `permutation`, `eig-preserved`)*
3. **Confirmed** — For the observed badly scaled matrix (no permutation) T was diagonal with powers of two on its diagonal (2^9, 2^3, 2^-4). *(cases: `two-outputs`)*
4. **Confirmed** — [S, P, B] returns a column scaling vector S (the diagonal of T) and a column permutation vector P ([1;2;3] when no permutation is applied). *(cases: `three-outputs`, `two-outputs`)*
5. **Confirmed** — An already balanced matrix ([1 2; 3 4]) and a scalar are returned unchanged. *(cases: `balanced`, `scalar`)*
6. **Confirmed** — 'noperm' gave the same B as the default for a matrix that needed no permutation. *(cases: `noperm`, `single-output`)*

## Edge cases (observed)

- *(non-normative)* Recorded for reference: A = [1 100 10000; 0.01 1 100; 0.0001 0.01 1] gave B = [1 1.5625 1.220703125; 0.64 1 0.78125; 0.8192 1.28 1] (within 1e-12); the lower-triangular [1 0 0; 2 3 0; 4 5 6] gave T equal to the reversal permutation and B = [6 5 4; 0 3 2; 0 0 1]. *(cases: `single-output`, `two-outputs`, `permutation`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A is not square. | `MATLAB:square` | `err-nonsquare` |
| A is sparse. | `MATLAB:balance:noBalanceSparse` | `err-sparse` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `single-output` | `double` | `[3, 3]` | [1 1.5625 1.220703125;0.64000000000000001 1 0.78125;0.81920000000000004 1.28 1] |
| `two-outputs` | `struct` | `[1, 1]` | T=[512 0 0;0 8 0;0 0 0.0625]; B=[1 1.5625 1.220703125;0.64000000000000001 1 0.78125;0.81920000000000004 1.28 1]; residual=0; T_is_diagonal=true; log2_diagT=[9 3 -4] |
| `eig-preserved` | `double` | `[1, 1]` | 0 |
| `balanced` | `double` | `[2, 2]` | [1 2;3 4] |
| `noperm` | `double` | `[3, 3]` | [1 1.5625 1.220703125;0.64000000000000001 1 0.78125;0.81920000000000004 1.28 1] |
| `permutation` | `struct` | `[1, 1]` | T=[0 0 1;0 1 0;1 0 0]; B=[6 5 4;0 3 2;0 0 1]; residual=0; T_is_diagonal=false; log2_diagT=[-Inf 0 -Inf] |
| `three-outputs` | `struct` | `[1, 1]` | S=[512;8;0.0625]; P=[1;2;3]; B=[1 1.5625 1.220703125;0.64000000000000001 1 0.78125;0.81920000000000004 1.28 1] |
| `scalar` | `double` | `[1, 1]` | 5 |
| `single-class` | `char` | `[1, 6]` | 'single' |
| `err-nonsquare` | — | — | error `MATLAB:square` |
| `err-sparse` | — | — | error `MATLAB:balance:noBalanceSparse` |

## Unresolved questions

- `balance.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
- `balance.q-exact-values` — Flagged for human review (constitution 8.2): whether conforming implementations must reproduce these exact balanced matrices (which depend on the balancing procedure) or only the similarity and scaling properties.
- `balance.q-noperm-permuting` — 'noperm' on a matrix that would otherwise be permuted was not exercised.
- `balance.q-complex` — Complex input was not exercised.
