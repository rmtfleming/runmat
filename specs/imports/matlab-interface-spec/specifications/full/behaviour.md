# `full` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/full_observations.json`](observations/full_observations.json)

## Summary

Convert a sparse matrix to full (dense) storage; a full input is returned in full storage.

## Confirmed invocation form

- `B = full(S)` — exercised in the experiment (case `from-sparse-matrix`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `from-sparse-matrix`, `from-sparse-vector`, `already-full`, `empty-sparse`)*
2. **Confirmed** — The output is full (issparse = false) and preserves the values of the input; an empty sparse input yields a 0-by-0 full array. *(cases: `from-sparse-matrix`, `from-sparse-vector`, `already-full`, `empty-sparse`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `from-sparse-matrix` | `double` | `[2, 2]` | `[1 0;0 2]` |
| `from-sparse-vector` | `double` | `[1, 3]` | `[0 3 0]` |
| `already-full` | `double` | `[2, 2]` | `[1 2;3 4]` |
| `empty-sparse` | `double` | `[0, 0]` | `[]` |

## Unresolved questions

- `full.q-class` — Confirm output class and issparse=false for all inputs.
- `full.q-complex` — Behaviour for complex sparse input (not tested).
