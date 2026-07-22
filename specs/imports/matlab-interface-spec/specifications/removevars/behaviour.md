# `removevars` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/removevars_observations.json`](observations/removevars_observations.json)

## Summary

Return a table with the specified variables removed.

## Confirmed invocation forms

- `T2 = removevars(T, vars)` — exercised in the experiment (case `by-name`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `table` for the observed inputs. *(cases: `by-name`)*
2. **Confirmed** — Returns a table with the named variable removed (removing one of three variables left a two-variable table); the row count is preserved. *(cases: `by-name`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `by-name` | `table` | `[1, 2]` | `<table [1 2]>` |

## Unresolved questions

- `removevars.q-index` — Removal by numeric index or logical mask.
- `removevars.q-multiple` — Removing multiple variables at once.
