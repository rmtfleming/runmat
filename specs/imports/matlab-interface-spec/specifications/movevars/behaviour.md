# `movevars` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/movevars_observations.json`](observations/movevars_observations.json)

## Summary

Return a table with the specified variables moved to a new position, before or after another variable.

## Confirmed invocation forms

- `T2 = movevars(T, vars, 'Before', location)` — exercised in the experiment (case `before`).

Additional forms are proposed but not yet exercised:
- `T2 = movevars(T, vars, 'After', location)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Returns a table with the variable moved to the requested position: moving 'c' before 'a' produced the variable order c,a,b (observed from the joined VariableNames); the variable and row counts are preserved. *(cases: `before`, `order`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `before` | `table` | `[1, 3]` | `<table [1 3]>` |
| `order` | `char` | `[1, 5]` | `'c,a,b'` |

## Unresolved questions

- `movevars.q-after` — The 'After' placement form.
- `movevars.q-index` — Numeric-index location.
