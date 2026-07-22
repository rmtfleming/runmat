# `normalize` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/normalize_observations.json`](observations/normalize_observations.json)

## Summary

Return a normalized version of the input; by default it centres and scales the data to zero mean and unit standard deviation.

## Confirmed invocation forms

- `N = normalize(A)` — exercised in the experiment (case `zscore`).
- `N = normalize(A, 'range')` — exercised in the experiment.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `zscore`, `range`)*
2. **Confirmed** — Default normalization is a z-score (zero mean, unit standard deviation): [1 2 3] gave [-1 0 1]; the 'range' method rescales to [0, 1]: [1 2 3] gave [0 0.5 1]. *(cases: `zscore`, `range`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `zscore` | `double` | `[1, 3]` | `[-1 0 1]` |
| `range` | `double` | `[1, 3]` | `[0 0.5 1]` |

## Unresolved questions

- `normalize.q-dim` — Operating dimension for matrices.
- `normalize.q-methods` — Other methods (norm, center, scale, medianiqr).
