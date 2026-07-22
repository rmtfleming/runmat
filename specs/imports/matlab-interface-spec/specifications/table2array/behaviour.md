# `table2array` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/table2array_observations.json`](observations/table2array_observations.json)

## Summary

Convert a table to a homogeneous array by horizontally concatenating its variables.

## Confirmed invocation form

- `A = table2array(T)` — exercised in the experiment (case `numeric`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `numeric`)*
2. **Confirmed** — Concatenates the table variables into a homogeneous array (a numeric table produced a double matrix). *(cases: `numeric`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `numeric` | `double` | `[2, 2]` | `[1 2;3 4]` |

## Unresolved questions

- `table2array.q-heterogeneous` — Behaviour when table variables have differing types or widths.
- `table2array.q-output-class` — Specify the resulting array class.
