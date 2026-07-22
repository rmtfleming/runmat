# `iscolumn` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/iscolumn_observations.json`](observations/iscolumn_observations.json)

## Summary

Return a logical scalar that is true when the input is a column vector (size N-by-1).

## Confirmed invocation forms

- `tf = iscolumn(A)` — exercised in the experiment (case `column`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `column`, `row`, `scalar`, `matrix`)*
2. **Confirmed** — Returns a 1-by-1 logical: true for an N-by-1 column (a scalar counts as a column), false for a row or a 2-D matrix. *(cases: `column`, `scalar`, `row`, `matrix`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `column` | `logical` | `[1, 1]` | `true` |
| `row` | `logical` | `[1, 1]` | `false` |
| `scalar` | `logical` | `[1, 1]` | `true` |
| `matrix` | `logical` | `[1, 1]` | `false` |

## Unresolved questions

- `iscolumn.q-empty` — Behaviour for empty inputs of various shapes.
