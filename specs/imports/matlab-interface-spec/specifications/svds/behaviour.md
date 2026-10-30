# `svds` — behaviour specification

- **Spec version:** 0.3.0
- **Status:** approved 2026-10-29 (Tier B widening W2, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/svds_observations.json`](observations/svds_observations.json)

## Summary

Return a subset of the singular values of a matrix (by default the largest few), and optionally the corresponding singular vectors.

## Confirmed invocation forms

- `s = svds(A)` — Largest singular values, default count. *(e.g. case `default-8`)*
- `s = svds(A, k)` — The k largest singular values. *(e.g. case `k`)*
- `s = svds(A, k, sigma)` — k singular values selected by `sigma`: 'largest', 'smallest', or a numeric target. *(e.g. case `smallest`)*
- `[U, S, V] = svds(___)` — Singular vectors and a diagonal matrix of singular values. *(e.g. case `usv-shapes`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is a full (non-sparse) column vector of class double for double input, sparse or full; for single input it is single. *(cases: `k`, `dense`, `single`)*
2. **Confirmed** — With k given, the k largest singular values are returned in descending order (diag([1 5 2 4]) with k=4 gave [5;4;2;1]); 'largest' gives the same selection. *(cases: `k`, `dense`, `order`, `largest`)*
3. **Confirmed** — Without k, an 8x8 matrix returned its 6 largest singular values and a 2x2 matrix returned both of its singular values. *(cases: `default-8`, `default-2`)*
4. **Confirmed** — k larger than min(size(A)) neither errored nor warned: k=3 for a 2x2 matrix returned its 2 singular values. *(cases: `k-too-large`, `warn-k-too-large`)*
5. **Confirmed** — An ordinary call (k=2 on a 2x2 matrix) issued no warning. *(cases: `warn-k`)*
6. **Confirmed** — k = 0 returns a 0x1 empty double. *(cases: `k-zero`)*
7. **Confirmed** — Rectangular input is accepted: a 2x3 matrix with k=2 gave [2;1]. *(cases: `rectangular`)*
8. **Confirmed** — sigma 'smallest' returns the smallest singular values (diag([4 1 3]), k=1 gave 1). *(cases: `smallest`)*
9. **Confirmed** — A numeric sigma returned the singular value closest to it (diag([4 1 3]), k=1, sigma 2.5 gave 3). *(cases: `sigma-numeric`)*
10. **Confirmed** — An all-zero matrix returns zeros (3x3 zero sparse, k=2 gave [0;0]). *(cases: `zero-matrix`)*
11. **Confirmed** — [U,S,V] returns U of size m-by-k, S of size k-by-k and V of size n-by-k, all full; diag(S) holds the same values as the single-output form; U and V have orthonormal columns and A*V equals U*S (norms below 1e-10). *(cases: `usv-shapes`, `usv-rect`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A is a char array. | `MATLAB:svds:InvalidAFloat` | `err-char` |
| k is negative. | `MATLAB:svds:InvalidK` | `err-k-negative` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `k` | `double` | `[2, 1]` | [3;2] |
| `default-8` | `double` | `[6, 1]` | [8;7;6;5;4;3] |
| `default-2` | `double` | `[2, 1]` | [3;2] |
| `k-too-large` | `double` | `[2, 1]` | [3;2] |
| `k-zero` | `double` | `[0, 1]` | zeros(0,1) |
| `dense` | `double` | `[2, 1]` | [4;3] |
| `rectangular` | `double` | `[2, 1]` | [2;1] |
| `smallest` | `double` | `[1, 1]` | 1 |
| `largest` | `double` | `[2, 1]` | [4;3] |
| `sigma-numeric` | `double` | `[1, 1]` | 3 |
| `zero-matrix` | `double` | `[2, 1]` | [0;0] |
| `single` | `single` | `[2, 1]` | [4;3] |
| `order` | `double` | `[4, 1]` | [5;4;2;1] |
| `usv-shapes` | `struct` | `[1, 1]` | size_U=[3 2]; size_S=[2 2]; size_V=[3 2]; issparse_S=false; issparse_U=false; issparse_V=false; diag_S=[4;3]; orth_U=0; orth_V=0; residual=0 |
| `usv-rect` | `struct` | `[1, 1]` | size_U=[2 2]; size_S=[2 2]; size_V=[3 2]; issparse_S=false; issparse_U=false; issparse_V=false; diag_S=[2;1]; orth_U=0; orth_V=0; residual=0 |
| `warn-k-too-large` | `struct` | `[1, 1]` | value=[3;2]; warning_id='' |
| `warn-k` | `struct` | `[1, 1]` | value=[3;2]; warning_id='' |
| `err-char` | — | — | error `MATLAB:svds:InvalidAFloat` |
| `err-k-negative` | — | — | error `MATLAB:svds:InvalidK` |

## Unresolved questions

- `svds.q-warning-coverage` — Warnings were captured only in the warn-* cases; other cases (including error and edge cases) were not checked for warnings, so an empty warnings list is not a complete statement.
- `svds.q-default-rule` — The default count was observed for 8x8 (6 values) and 2x2 (2 values) only; the general rule was not exercised further.
- `svds.q-sigma-numeric` — Numeric sigma was exercised with one target only; tie-breaking and multiple values near the target were not exercised.
- `svds.q-options` — Options (an options struct or name-value settings such as tolerance or iteration limits) and complex input were not exercised.
- `svds.q-vector-signs` — Singular-vector signs are not compared; only shapes, orthonormality and the A*V = U*S relation were checked.

Resolved in 0.3.0: `svds.q-default`, `svds.q-vectors`.
