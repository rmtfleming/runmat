# `histc` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/histc_observations.json`](observations/histc_observations.json)

## Summary

Count the number of values that fall into each of a set of bins defined by edge boundaries (a legacy function superseded by histcounts).

## Confirmed invocation forms

- `n = histc(data, edges)` — exercised in the experiment (case `basic`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `basic`)*
2. **Confirmed** — Counts the values falling into each bin defined by the edges; [1 2 2 3] with edges [1 2 3] gave [1 2 1], the final bin counting values equal to the last edge. *(cases: `basic`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `basic` | `double` | `[1, 3]` | `[1 2 1]` |

## Unresolved questions

- `histc.q-deprecated` — Confirm any deprecation notice relative to histcounts.
