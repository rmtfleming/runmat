# `outerjoin` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/outerjoin_observations.json`](observations/outerjoin_observations.json)

## Summary

Combine two tables by matching a common key variable.

## Confirmed invocation forms

- `T = outerjoin(Tleft, Tright)` — exercised in the experiment (case `basic`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `table` for the observed inputs. *(cases: `basic`)*
2. **Confirmed** — An outer join of two 2-row tables sharing a common key variable produced a 3-row, 4-variable table. The specific join semantics (which rows are kept and any fill values) are not asserted here. *(cases: `basic`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `basic` | `table` | `[3, 4]` | `<table [3 4]>` |

## Unresolved questions

- `outerjoin.q-keys` — The 'Keys'/'LeftKeys'/'RightKeys' options.
- `outerjoin.q-type` — Left/right/full outer join variants and fill values.
