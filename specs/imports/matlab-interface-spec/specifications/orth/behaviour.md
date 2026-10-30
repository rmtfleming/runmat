# `orth` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/orth_observations.json`](observations/orth_observations.json)

## Summary

Return a matrix with orthonormal columns spanning the range (column space) of the input; for the observed inputs the number of columns equalled the rank of the input.

## Confirmed invocation forms

- `Q = orth(A)` — Orthonormal basis of range(A). *(e.g. case `full-rank`)*
- `Q = orth(A, t)` — A second numeric argument is accepted and changed the number of basis columns in the one case observed. *(e.g. case `second-arg`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Q had size(A,1) rows and as many columns as the rank of A: 3x3 full rank gave 3x3, rank 2 gave 3x2, rank 1 gave 3x1, zeros(3) gave 3x0, a 4x2 full-column-rank matrix gave 4x2, a 2x4 rank-2 matrix gave 2x2. *(cases: `full-rank`, `rank2`, `rank1`, `zero`, `tall`, `wide`)*
2. **Confirmed** — norm(Q'*Q - I) and the span residual norm(A - Q*(Q'*A)) were at rounding level (below 1.2e-14 for double, below 5e-6 for single), including for complex input. *(cases: `full-rank`, `rank2`, `rank1`, `tall`, `wide`, `single`, `complex`)*
3. **Confirmed** — Single input gives single output; complex double input gave a double result. *(cases: `single`, `complex`)*
4. **Confirmed** — orth([]) gives 0x0 and orth(zeros(3,0)) gives 3x0. *(cases: `empty`, `empty-3x0`)*
5. **Confirmed** — orth(diag([1 1e-3 0]), 1e-2) returned a 3x1 basis (the 1e-3 direction dropped, span residual 1e-3). *(cases: `second-arg`)*

## Edge cases (observed)

- *(non-normative)* Observed for the one input diag([2 0 5]): abs(Q) was [0 1; 0 0; 1 0] (first column +/-e3, second +/-e1; signs not recorded). Column order is not required. *(cases: `diag-abs`)*
- *(non-normative)* Inference (not observed): an orthonormal basis of a range is not unique (signs, and rotations within repeated singular values), so only size, orthonormality and span should be required of an implementation. *(cases: `full-rank`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A is sparse. | `MATLAB:svd:noSVDsparse` | `err-sparse` |
| A is an integer or char array. | `MATLAB:svd:inputType` | `err-int`, `err-char` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `full-rank` | `struct` | `[1, 1]` | size=[3 3]; class='double'; orthonormal_residual=7.2427647352403656e-16; span_residual=1.1042951495254651e-14; rank_Q=3 |
| `rank2` | `struct` | `[1, 1]` | size=[3 2]; class='double'; orthonormal_residual=3.943111286301344e-16; span_residual=3.0855283919595267e-15; rank_Q=2 |
| `rank1` | `struct` | `[1, 1]` | size=[3 1]; class='double'; orthonormal_residual=5.5511151231257827e-16; span_residual=1.6653345369377348e-15; rank_Q=1 |
| `zero` | `struct` | `[1, 1]` | size=[3 0]; class='double'; orthonormal_residual=0; span_residual=0; rank_Q=0 |
| `tall` | `struct` | `[1, 1]` | size=[4 2]; class='double'; orthonormal_residual=2.6803154833089315e-16; span_residual=4.7850970656354905e-16; rank_Q=2 |
| `wide` | `struct` | `[1, 1]` | size=[2 2]; class='double'; orthonormal_residual=2.2204460492503131e-16; span_residual=1.602108806848966e-15; rank_Q=2 |
| `diag-abs` | `double` | `[3, 2]` | [0 1;0 0;1 0] |
| `empty` | `double` | `[0, 0]` | [] |
| `empty-3x0` | `double` | `[3, 0]` | zeros(3,0) |
| `single` | `struct` | `[1, 1]` | size=[3 3]; class='single'; orthonormal_residual=2.7298972327116644e-07; span_residual=4.1370994949829765e-06; rank_Q=3 |
| `complex` | `struct` | `[1, 1]` | size=[2 2]; class='double'; orthonormal_residual=5.3606309666178639e-16; span_residual=8.951979685313743e-16; rank_Q=2 |
| `second-arg` | `struct` | `[1, 1]` | size=[3 1]; class='double'; orthonormal_residual=0; span_residual=0.001; rank_Q=1 |
| `err-sparse` | — | — | error `MATLAB:svd:noSVDsparse` |
| `err-int` | — | — | error `MATLAB:svd:inputType` |
| `err-char` | — | — | error `MATLAB:svd:inputType` |

## Unresolved questions

- `orth.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `orth.q-default-tol` — The default rank threshold was not measured; only clearly separated singular values were exercised.
- `orth.q-second-arg` — Whether the second argument is an absolute threshold or relative to the largest singular value is not established: the one case had largest singular value 1.
- `orth.q-error-ids` — The error identifiers name another function (svd); whether an implementation must reproduce them is a human-review question (constitution 8.2).
