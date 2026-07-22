# `getReport` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/getReport_observations.json`](observations/getReport_observations.json)

## Summary

Return a formatted text report describing an exception object.

## Confirmed invocation forms

- `report = getReport(ME)` — exercised in the experiment (case `basic`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `char` for the observed inputs. *(cases: `basic`)*
2. **Confirmed** — Returns char report text describing the exception; for a plain MException the observed report was the message text 'oops'. *(cases: `basic`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `basic` | `char` | `[1, 4]` | `'oops'` |

## Unresolved questions

- `getReport.q-options` — The 'extended'/'basic' and hyperlink options are not exercised.
