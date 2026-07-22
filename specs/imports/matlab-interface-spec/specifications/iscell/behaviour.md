# `iscell` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/iscell_observations.json`](observations/iscell_observations.json)

## Summary

Return a logical scalar that is true when the input is a cell array and false otherwise.

## Confirmed invocation form

- `tf = iscell(A)` — exercised in the experiment (case `cell-array`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `cell-array`, `numeric-scalar`, `char-row`, `empty-cell`, `struct`)*
2. **Confirmed** — Returns a 1-by-1 logical: true for cell arrays including the empty cell {}, and false for numeric, char, and struct inputs. *(cases: `cell-array`, `empty-cell`, `numeric-scalar`, `char-row`, `struct`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `cell-array` | `logical` | `[1, 1]` | `true` |
| `numeric-scalar` | `logical` | `[1, 1]` | `false` |
| `char-row` | `logical` | `[1, 1]` | `false` |
| `empty-cell` | `logical` | `[1, 1]` | `true` |
| `struct` | `logical` | `[1, 1]` | `false` |

## Unresolved questions

- `iscell.q-output` — Confirm output class (logical) and size (1x1) across inputs.
