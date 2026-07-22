# `spalloc` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/spalloc_observations.json`](observations/spalloc_observations.json)

## Summary

Allocate space for a sparse matrix of a given size with room for a stated number of nonzero elements; the result is initially all zeros.

## Confirmed invocation forms

- `S = spalloc(m, n, nz)` — exercised in the experiment (case `basic`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `basic`)*
2. **Confirmed** — Returns an all-zero sparse matrix of the requested size (issparse is true; the observed 3-by-3 result had no stored nonzeros). *(cases: `basic`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `basic` | `double` | `[3, 3]` | `sparse(zeros(0,1), zeros(0,1), zeros(0,1), 3, 3)` |

## Unresolved questions

- None recorded.
