# `array2table` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/array2table_observations.json`](observations/array2table_observations.json)

## Summary

Convert a matrix to a table, with one table variable per column of the matrix.

## Confirmed invocation form

- `T = array2table(A)` — exercised in the experiment (case `basic`).

Additional forms are proposed but not yet exercised:
- `T = array2table(A, 'VariableNames', names)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `table` for the observed inputs. *(cases: `basic`, `varnames`)*
2. **Confirmed** — The output is a table with one variable per column of the input matrix; the row count is preserved. *(cases: `basic`, `varnames`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `basic` | `table` | `[2, 3]` | `<table [2 3]>` |
| `varnames` | `table` | `[2, 2]` | `<table [2 2]>` |

## Unresolved questions

- `array2table.q-table-support` — Requires table type support; specify default variable names and column types.
- `array2table.q-rownames` — Specify the 'RowNames' name-value option.
