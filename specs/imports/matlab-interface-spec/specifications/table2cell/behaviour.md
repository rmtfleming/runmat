# `table2cell` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/table2cell_observations.json`](observations/table2cell_observations.json)

## Summary

Convert a table to a cell array, preserving the row-by-variable layout.

## Confirmed invocation form

- `C = table2cell(T)` — exercised in the experiment (case `basic`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `cell` for the observed inputs. *(cases: `basic`)*
2. **Confirmed** — Returns a cell array with the table's row-by-variable layout. *(cases: `basic`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `basic` | `cell` | `[2, 2]` | `<cell [2 2]>` |

## Unresolved questions

- `table2cell.q-table-support` — Requires table type support; specify cell layout and element types.
