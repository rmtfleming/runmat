# `bounds` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/bounds_observations.json`](observations/bounds_observations.json)

## Summary

Return the smallest and largest elements of an array, optionally along a specified dimension.

## Confirmed invocation form

- `[lo, hi] = bounds(A)` — exercised in the experiment (case `vector`).

Additional forms are proposed but not yet exercised:
- `[lo, hi] = bounds(A, dim)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `vector`, `matrix-default`, `matrix-dim2`)*
2. **Confirmed** — The first output is the minimum: a vector yields a scalar; for a matrix the default is the column-wise minimum (1-by-n) and dim = 2 gives the row-wise minimum (m-by-1). *(cases: `vector`, `matrix-default`, `matrix-dim2`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `vector` | `double` | `[1, 1]` | `1` |
| `matrix-default` | `double` | `[1, 2]` | `[1 2]` |
| `matrix-dim2` | `double` | `[2, 1]` | `[1;3]` |

## Unresolved questions

- `bounds.q-second-output` — Specify the second output (hi); the harness records only the first output (lo).
- `bounds.q-nan` — NaN handling and the 'omitnan'/'includenan' options.
