# `colamd` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/colamd_observations.json`](observations/colamd_observations.json)

## Summary

Return a column permutation vector that tends to reduce fill-in when factorizing a sparse matrix.

## Confirmed invocation forms

- `p = colamd(S)` — exercised in the experiment (case `basic`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `basic`)*
2. **Confirmed** — Returns a length-n row vector that is a permutation of 1..n (n=3 gave a permutation of {1,2,3}); the specific ordering is algorithm-dependent and is not asserted. *(cases: `basic`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `basic` | `double` | `[1, 3]` | `[2 1 3]` |

## Unresolved questions

- `colamd.q-permutation` — Confirm the output is a permutation of 1..n and its orientation.
- `colamd.q-values` — The specific permutation is algorithm-dependent; treat as non-normative.
