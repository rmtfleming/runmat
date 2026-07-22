# `license` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/license_observations.json`](observations/license_observations.json)

## Summary

Query licensing: with 'test' and a feature name, return whether that feature is licensed.

## Confirmed invocation forms

- `result = license('test', feature)` — exercised in the experiment (case `test-matlab`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `test-matlab`)*
2. **Confirmed** — license('test', feature) returns a numeric (double) result: 1 when the feature is licensed (observed for MATLAB). *(cases: `test-matlab`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `test-matlab` | `double` | `[1, 1]` | `1` |

## Unresolved questions

- `license.q-other-actions` — The no-argument and 'inuse'/'checkout' forms are not exercised.
- `license.q-value` — The license-number form returns installation-specific text; not exercised.
