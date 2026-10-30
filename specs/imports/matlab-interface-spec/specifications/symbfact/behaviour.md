# `symbfact` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/symbfact_observations.json`](observations/symbfact_observations.json)

## Summary

Return the column counts of the Cholesky factor predicted from a sparsity pattern, optionally with the tree height, parent vector, postorder and the factor's pattern.

## Confirmed invocation forms

- `count = symbfact(A)` — Column counts of the predicted Cholesky factor of A. *(e.g. case `tridiagonal`)*
- `count = symbfact(A, 'col')` — Column counts for A'*A. *(e.g. case `col`)*
- `[count, h, parent, post, R] = symbfact(___)` — Also height, elimination tree, postorder and factor pattern. *(e.g. case `outputs`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — count is a 1-by-n double row vector. *(cases: `tridiagonal`, `class`)*
2. **Confirmed** — count(j) is the number of nonzeros in row j of the returned upper-triangular factor pattern R (column j of R'), including the diagonal: 4x4 tridiagonal gave [2 2 2 1]; the arrow matrix with the dense row/column first gave [4 3 2 1] (fill-in); with the dense row/column last it gave [2 2 2 1]; the 3x3 diagonal gave [1 1 1]; the 5x5 matrix with corner entries gave [3 3 3 2 1]. *(cases: `tridiagonal`, `arrow`, `arrow-reversed`, `diag`, `class`, `outputs`)*
3. **Confirmed** — For the 4x4 arrow matrix the extra outputs were h = 4, parent = [2 3 4 0], post = [1 2 3 4], and R a sparse upper-triangular all-ones pattern. *(cases: `outputs`)*
4. **Confirmed** — Full input gave the same counts as sparse input. *(cases: `dense-input`, `tridiagonal`)*
5. **Confirmed** — 'col' accepts a non-square matrix: [1 0; 1 1; 0 1] gave [2 1]. *(cases: `col`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A is not square and 'col' is not given. | `MATLAB:symbfact:InvalidInput` | `err-nonsquare` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `tridiagonal` | `double` | `[1, 4]` | [2 2 2 1] |
| `arrow` | `double` | `[1, 4]` | [4 3 2 1] |
| `arrow-reversed` | `double` | `[1, 4]` | [2 2 2 1] |
| `diag` | `double` | `[1, 3]` | [1 1 1] |
| `outputs` | `struct` | `[1, 1]` | count=[4 3 2 1]; height=4; parent=[2 3 4 0]; post=[1 2 3 4]; R=[1 1 1 1;0 1 1 1;0 0 1 1;0 0 0 1]; issparse_R=true |
| `col` | `double` | `[1, 2]` | [2 1] |
| `dense-input` | `double` | `[1, 4]` | [2 2 2 1] |
| `class` | `double` | `[1, 5]` | [3 3 3 2 1] |
| `err-nonsquare` | — | — | error `MATLAB:symbfact:InvalidInput` |

## Unresolved questions

- `symbfact.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
- `symbfact.q-more-options` — Option values other than 'col', and forms with more input arguments, were not exercised.
