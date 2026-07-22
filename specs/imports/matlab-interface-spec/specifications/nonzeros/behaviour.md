# `nonzeros` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/nonzeros_observations.json`](observations/nonzeros_observations.json)

## Summary

Return the nonzero elements of the input as a column vector, in column-major order.

## Confirmed invocation forms

- `v = nonzeros(A)` — exercised in the experiment (case `row`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `row`, `matrix`, `col-major`, `sparse`)*
2. **Confirmed** — Returns the nonzero elements as a column vector in column-major order: nonzeros([1 2; 3 0]) gave [1;3;2] (column-major traversal 1,3,2,0; row-major would give [1;2;3]), which distinguishes the two orderings. Works for full and sparse inputs. *(cases: `row`, `matrix`, `col-major`, `sparse`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `row` | `double` | `[2, 1]` | `[1;2]` |
| `matrix` | `double` | `[3, 1]` | `[1;2;4]` |
| `col-major` | `double` | `[3, 1]` | `[1;3;2]` |
| `sparse` | `double` | `[1, 1]` | `3` |

## Unresolved questions

- None recorded.
