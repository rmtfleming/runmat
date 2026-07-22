# `isrow` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isrow_observations.json`](observations/isrow_observations.json)

## Summary

Return a logical scalar that is true when the input is a row vector (size 1-by-N).

## Confirmed invocation forms

- `tf = isrow(A)` — exercised in the experiment (case `row`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `row`, `column`, `scalar`, `matrix`)*
2. **Confirmed** — Returns a 1-by-1 logical: true for a 1-by-N row (a scalar counts as a row), false for a column or a 2-D matrix. *(cases: `row`, `scalar`, `column`, `matrix`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `row` | `logical` | `[1, 1]` | `true` |
| `column` | `logical` | `[1, 1]` | `false` |
| `scalar` | `logical` | `[1, 1]` | `true` |
| `matrix` | `logical` | `[1, 1]` | `false` |

## Unresolved questions

- `isrow.q-empty` — Behaviour for empty inputs of various shapes.
