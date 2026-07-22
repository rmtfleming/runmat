# `verLessThan` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/verLessThan_observations.json`](observations/verLessThan_observations.json)

## Summary

Return whether an installed toolbox version is older than a specified version string (a legacy function).

## Confirmed invocation forms

- `tf = verLessThan(toolbox, version)` — exercised in the experiment (case `older`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `older`, `newer`)*
2. **Confirmed** — Returns a 1-by-1 logical: comparing MATLAB against a very high version gave true and against a very low version gave false. *(cases: `older`, `newer`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `older` | `logical` | `[1, 1]` | `true` |
| `newer` | `logical` | `[1, 1]` | `false` |

## Unresolved questions

- `verLessThan.q-deprecated` — Confirm any deprecation notice relative to isMATLABReleaseOlderThan.
