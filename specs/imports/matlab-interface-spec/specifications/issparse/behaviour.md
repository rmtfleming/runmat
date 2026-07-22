# `issparse` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/issparse_observations.json`](observations/issparse_observations.json)

## Summary

Return a logical scalar that is true when the input uses sparse storage and false otherwise.

## Confirmed invocation forms

- `tf = issparse(A)` — exercised in the experiment (case `sparse`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `sparse`, `full`, `empty-sparse`)*
2. **Confirmed** — Returns a 1-by-1 logical: true for sparse-storage inputs including an empty sparse array, false for full inputs. *(cases: `sparse`, `empty-sparse`, `full`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `sparse` | `logical` | `[1, 1]` | `true` |
| `full` | `logical` | `[1, 1]` | `false` |
| `empty-sparse` | `logical` | `[1, 1]` | `true` |

## Unresolved questions

- `issparse.q-output` — Confirm output class (logical) and size (1x1).
