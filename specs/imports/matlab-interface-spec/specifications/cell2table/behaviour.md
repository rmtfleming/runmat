# `cell2table` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/cell2table_observations.json`](observations/cell2table_observations.json)

## Summary

Convert a cell array to a table, with one table variable per column of the cell array.

## Confirmed invocation form

- `T = cell2table(C)` — exercised in the experiment (case `basic`).

Additional forms are proposed but not yet exercised:
- `T = cell2table(C, 'VariableNames', names)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `table` for the observed inputs. *(cases: `basic`, `varnames`)*
2. **Confirmed** — The output is a table with one variable per column of the cell array; the row count is preserved. *(cases: `basic`, `varnames`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `basic` | `table` | `[2, 2]` | `<table [2 2]>` |
| `varnames` | `table` | `[1, 2]` | `<table [1 2]>` |

## Unresolved questions

- `cell2table.q-table-support` — Requires table type support; specify default variable names and per-column types.
- `cell2table.q-nested` — Behaviour with nested cells / non-scalar cell contents.
