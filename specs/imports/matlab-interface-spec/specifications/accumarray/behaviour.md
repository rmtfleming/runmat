# `accumarray` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/accumarray_observations.json`](observations/accumarray_observations.json)

## Summary

Accumulate values into an output array at positions given by subscripts; by default it sums values that map to the same position.

## Confirmed invocation forms

- `A = accumarray(subs, vals)` — exercised in the experiment (case `sum`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `sum`)*
2. **Confirmed** — Accumulates values into a column vector indexed by subscripts, summing values that share a subscript by default (subs [1;2;1], vals [10;20;30] gave [40;20]). *(cases: `sum`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `sum` | `double` | `[2, 1]` | `[40;20]` |

## Unresolved questions

- `accumarray.q-func` — The custom accumulation-function form.
- `accumarray.q-size` — The size and fill-value options.
