# `addvars` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/addvars_observations.json`](observations/addvars_observations.json)

## Summary

Return a table with one or more new variables appended.

## Confirmed invocation forms

- `T2 = addvars(T, var)` — exercised in the experiment (case `append`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `table` for the observed inputs. *(cases: `append`)*
2. **Confirmed** — Returns a table with the new variable appended (appending to a one-variable table gave a two-variable table); the row count is preserved. *(cases: `append`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `append` | `table` | `[2, 2]` | `<table [2 2]>` |

## Unresolved questions

- `addvars.q-position` — The 'Before'/'After' name-value options for placement.
- `addvars.q-names` — Specifying NewVariableNames.
