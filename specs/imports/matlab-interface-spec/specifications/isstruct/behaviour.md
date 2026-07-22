# `isstruct` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isstruct_observations.json`](observations/isstruct_observations.json)

## Summary

Return a logical scalar that is true when the input is a structure array and false otherwise.

## Confirmed invocation form

- `tf = isstruct(A)` — exercised in the experiment (case `struct`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `struct`, `struct-array`, `numeric`, `cell`)*
2. **Confirmed** — Returns a 1-by-1 logical: true for scalar structs and struct arrays, false for numeric and cell inputs. *(cases: `struct`, `struct-array`, `numeric`, `cell`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `struct` | `logical` | `[1, 1]` | `true` |
| `struct-array` | `logical` | `[1, 1]` | `true` |
| `numeric` | `logical` | `[1, 1]` | `false` |
| `cell` | `logical` | `[1, 1]` | `false` |

## Unresolved questions

- `isstruct.q-output` — Confirm output class (logical) and size (1x1).
