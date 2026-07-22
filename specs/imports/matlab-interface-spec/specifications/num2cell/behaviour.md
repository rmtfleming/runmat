# `num2cell` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/num2cell_observations.json`](observations/num2cell_observations.json)

## Summary

Convert an array to a cell array whose elements are the elements of the input; an optional dimension argument groups elements along the specified dimensions.

## Confirmed invocation form

- `C = num2cell(A)` — exercised in the experiment (case `vector`).

Additional forms are proposed but not yet exercised:
- `C = num2cell(A, dim)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `cell` for the observed inputs. *(cases: `vector`, `matrix`, `dim1`, `scalar`)*
2. **Confirmed** — Returns a cell array; by default one cell per element preserving the input shape, and a dimension argument groups elements along that dimension (dim = 1 on a 2-by-2 matrix produced a 1-by-2 cell). *(cases: `vector`, `matrix`, `dim1`, `scalar`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `vector` | `cell` | `[1, 3]` | `<cell [1 3]>` |
| `matrix` | `cell` | `[2, 2]` | `<cell [2 2]>` |
| `dim1` | `cell` | `[1, 2]` | `<cell [1 2]>` |
| `scalar` | `cell` | `[1, 1]` | `<cell [1 1]>` |

## Unresolved questions

- `num2cell.q-shape` — Confirm cell array shape for each form, especially with dim.
