# `istable` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/istable_observations.json`](observations/istable_observations.json)

## Summary

Return a logical scalar that is true when the input is a table.

## Confirmed invocation forms

- `tf = istable(A)` — exercised in the experiment (case `table`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `table`, `numeric`, `cell`)*
2. **Confirmed** — Returns a 1-by-1 logical: true for a table, false for numeric and cell inputs. *(cases: `table`, `numeric`, `cell`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `table` | `logical` | `[1, 1]` | `true` |
| `numeric` | `logical` | `[1, 1]` | `false` |
| `cell` | `logical` | `[1, 1]` | `false` |

## Unresolved questions

- `istable.q-timetable` — Behaviour for a timetable input.
