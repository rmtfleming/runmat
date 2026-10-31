# `sprank` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N5, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/sprank_observations.json`](observations/sprank_observations.json)

## Summary

Return the structural rank of a matrix, which depends only on its nonzero pattern; all observed values equal the largest number of nonzeros with no two in the same row or column.

## Confirmed invocation forms

- `r = sprank(A)` — Structural rank (COBRA form, A sparse). *(e.g. case `identity`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Only the nonzero pattern counts: the all-ones 2x2 matrix has sprank 2 while its rank is 1. *(cases: `cancelling`)*
2. **Confirmed** — sprank(speye(3)) = 3; a zero row reduced it to 2; [1 1 0; 0 0 1; 0 0 1] gave 2; the 2x3 [1 1 0; 0 1 1] and the 4x2 [1 0; 1 0; 1 0; 0 1] each gave 2. *(cases: `identity`, `zero-row`, `matching-limited`, `wide`, `tall`)*
3. **Confirmed** — An all-zero sparse matrix and a 0x0 sparse matrix gave 0. *(cases: `zero-matrix`, `empty`)*
4. **Confirmed** — Full, sparse logical and sparse complex inputs are accepted and give the structural rank (2 for each 2x2 diagonal case). *(cases: `dense`, `logical`, `complex`)*
5. **Confirmed** — The result is a double scalar. *(cases: `identity`, `dense`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A is char or a cell array. | `MATLAB:math:mustBeFloatOrLogical` | `err-char`, `err-cell` |
| No input is given. | `MATLAB:minrhs` | `err-no-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `identity` | `double` | `[1, 1]` | 3 |
| `cancelling` | `double` | `[1, 2]` | [2 1] |
| `zero-row` | `double` | `[1, 1]` | 2 |
| `matching-limited` | `double` | `[1, 1]` | 2 |
| `wide` | `double` | `[1, 1]` | 2 |
| `tall` | `double` | `[1, 1]` | 2 |
| `zero-matrix` | `double` | `[1, 1]` | 0 |
| `empty` | `double` | `[1, 1]` | 0 |
| `dense` | `double` | `[1, 1]` | 2 |
| `logical` | `double` | `[1, 1]` | 2 |
| `complex` | `double` | `[1, 1]` | 2 |
| `err-char` | — | — | error `MATLAB:math:mustBeFloatOrLogical` |
| `err-cell` | — | — | error `MATLAB:math:mustBeFloatOrLogical` |
| `err-no-arg` | — | — | error `MATLAB:minrhs` |

## Unresolved questions

- `sprank.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `sprank.q-error-ids` — The char/cell identifier is a generic validator identifier (MATLAB:math:mustBeFloatOrLogical); whether an implementation must reproduce it is a human-review question (constitution 8.2).
