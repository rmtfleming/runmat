# `methods` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/methods_observations.json`](observations/methods_observations.json)

## Summary

Return the method names of an object or class as a cell array of character vectors.

## Confirmed invocation forms

- `c = methods(A)` — exercised in the experiment (case `object`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Returns an N-by-1 cell array whose elements are char vectors (verified with iscellstr); inputParser gave 19 names. The count and names are class-dependent. *(cases: `object`, `elem-type`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `object` | `cell` | `[19, 1]` | `<cell [19 1]>` |
| `elem-type` | `logical` | `[1, 1]` | `true` |

## Unresolved questions

- `methods.q-class-dependent` — The specific names depend on the class.
- `methods.q-full` — The '-full' option is not exercised.
