# `seconds` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/seconds_observations.json`](observations/seconds_observations.json)

## Summary

Convert a number to a duration in seconds, or convert a duration to a number of seconds.

## Confirmed invocation forms

- `d = seconds(x)` — exercised in the experiment (case `to-duration`).

Additional forms are proposed but not yet exercised:
- `n = seconds(d)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `duration` for the observed inputs. *(cases: `to-duration`)*
2. **Confirmed** — seconds(5) returns a duration object (class duration). *(cases: `to-duration`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `to-duration` | `duration` | `[1, 1]` | `<duration [1 1]>` |

## Unresolved questions

- `seconds.q-from-duration` — The duration-to-number form is not exercised.
