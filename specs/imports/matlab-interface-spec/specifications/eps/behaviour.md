# `eps` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/eps_observations.json`](observations/eps_observations.json)

## Summary

Return the floating-point relative accuracy: the distance from 1.0 to the next larger double, or the spacing for a given value or class.

## Confirmed invocation forms

- `d = eps` — exercised in the experiment (case `default`).
- `d = eps(classname)` — exercised in the experiment.

Additional forms are proposed but not yet exercised:
- `d = eps(x)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — eps returns the double spacing near 1.0, 2.2204460492503131e-16; eps('single') returns the single spacing 1.1920928955078125e-07. *(cases: `default`, `single-class`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `default` | `double` | `[1, 1]` | `2.2204460492503131e-16` |
| `single-class` | `single` | `[1, 1]` | `1.1920928955078125e-07` |

## Unresolved questions

- `eps.q-value-form` — The eps(x) value-dependent form is not exercised.
