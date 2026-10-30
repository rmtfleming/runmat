# `etree` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/etree_observations.json`](observations/etree_observations.json)

## Summary

Return the elimination tree of a square symmetric sparsity pattern (or of A'*A with 'col') as a parent vector, optionally with a postorder permutation.

## Confirmed invocation forms

- `p = etree(A)` — Elimination tree of A. *(e.g. case `tridiagonal`)*
- `p = etree(A, 'sym')` — Same as the default. *(e.g. case `sym`)*
- `p = etree(A, 'col')` — Column elimination tree (tree of A'*A). *(e.g. case `col`)*
- `[p, q] = etree(___)` — Also a postorder permutation of the tree. *(e.g. case `postorder`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is a 1-by-n double row vector. *(cases: `tridiagonal`, `diag`, `pentadiag`)*
2. **Confirmed** — p(j) is the parent of column j in the elimination tree and 0 marks a root: the 4x4 tridiagonal and arrow matrices both gave [2 3 4 0]; the 5x5 matrix with corner entries gave [2 3 4 5 0]; a diagonal matrix gave all zeros. *(cases: `tridiagonal`, `arrow`, `pentadiag`, `diag`)*
3. **Confirmed** — Only the upper triangle is used: triu of the tridiagonal matrix gave the same tree [2 3 4 0], while tril gave [0 0 0 0]. *(cases: `upper-only`, `lower-only`, `tridiagonal`)*
4. **Confirmed** — 'sym' gives the same result as the default. *(cases: `sym`, `tridiagonal`)*
5. **Confirmed** — Full input gave the same tree as sparse input. *(cases: `dense-input`, `tridiagonal`)*
6. **Confirmed** — 'col' accepts a non-square matrix and returns the tree of its columns: the 3x2 matrix [1 0; 1 1; 0 1] gave [2 0]. *(cases: `col`)*
7. **Confirmed** — The second output is a 1-by-n permutation; it was [1 2 3 4] for the observed chain trees and for the branching tree [3 3 4 0] (each a valid postorder of its tree). *(cases: `postorder`, `postorder-arrow`, `postorder-branching`)*
8. **Confirmed** — A matrix whose columns 1 and 2 both connect only to column 3 gave parent [3 3 4 0]. *(cases: `postorder-branching`)*
9. **Confirmed** — 0x0 input gives a 1x0 empty double. *(cases: `empty`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A is not square and 'col' is not given. | `MATLAB:etree:InvalidInput` | `err-nonsquare` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `tridiagonal` | `double` | `[1, 4]` | [2 3 4 0] |
| `arrow` | `double` | `[1, 4]` | [2 3 4 0] |
| `diag` | `double` | `[1, 3]` | [0 0 0] |
| `pentadiag` | `double` | `[1, 5]` | [2 3 4 5 0] |
| `postorder` | `struct` | `[1, 1]` | out1=[2 3 4 0]; out2=[1 2 3 4] |
| `postorder-arrow` | `struct` | `[1, 1]` | out1=[2 3 4 0]; out2=[1 2 3 4] |
| `col` | `double` | `[1, 2]` | [2 0] |
| `sym` | `double` | `[1, 4]` | [2 3 4 0] |
| `dense-input` | `double` | `[1, 4]` | [2 3 4 0] |
| `upper-only` | `double` | `[1, 4]` | [2 3 4 0] |
| `lower-only` | `double` | `[1, 4]` | [0 0 0 0] |
| `postorder-branching` | `struct` | `[1, 1]` | out1=[3 3 4 0]; out2=[1 2 3 4] |
| `empty` | `double` | `[1, 0]` | zeros(1,0) |
| `err-nonsquare` | — | — | error `MATLAB:etree:InvalidInput` |

## Unresolved questions

- `etree.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
- `etree.q-postorder-general` — Postorders that differ from the identity were not observed, so the order among siblings is not specified.
