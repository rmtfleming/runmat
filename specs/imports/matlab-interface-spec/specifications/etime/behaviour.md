# `etime` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/etime_observations.json`](observations/etime_observations.json)

## Summary

Return the number of seconds between two date vectors (a legacy function).

## Confirmed invocation forms

- `s = etime(t2, t1)` — exercised in the experiment (case `one-second`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `one-second`)*
2. **Confirmed** — Returns the number of seconds between two date vectors as a double (1 for vectors one second apart). *(cases: `one-second`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `one-second` | `double` | `[1, 1]` | `1` |

## Unresolved questions

- `etime.q-deprecated` — Confirm any deprecation notice.
- `etime.q-negative` — Behaviour when t2 is earlier than t1.
