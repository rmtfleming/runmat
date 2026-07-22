# `isobject` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isobject_observations.json`](observations/isobject_observations.json)

## Summary

Return a logical scalar that is true when the input is an object of a class that is not one of MATLAB's fundamental types.

## Confirmed invocation forms

- `tf = isobject(A)` — exercised in the experiment (case `object`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `object`, `numeric`, `char`, `cell`)*
2. **Confirmed** — Returns a 1-by-1 logical: true for an object instance (an inputParser), false for numeric, char, and cell inputs. *(cases: `object`, `numeric`, `char`, `cell`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `object` | `logical` | `[1, 1]` | `true` |
| `numeric` | `logical` | `[1, 1]` | `false` |
| `char` | `logical` | `[1, 1]` | `false` |
| `cell` | `logical` | `[1, 1]` | `false` |

## Unresolved questions

- `isobject.q-fundamental` — Confirm which built-in classes count as non-objects.
