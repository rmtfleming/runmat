# `fields` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/fields_observations.json`](observations/fields_observations.json)

## Summary

Return the field names of a structure as a cell array of character vectors (a legacy equivalent of fieldnames). (Documented as not recommended / legacy; verify by observation.)

## Confirmed invocation form

- `c = fields(s)` — exercised in the experiment (case `two-fields`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `cell` for the observed inputs. *(cases: `two-fields`, `empty-struct`)*
2. **Confirmed** — Returns an N-by-1 cell array of field-name char vectors (0-by-1 for a struct with no fields). *(cases: `two-fields`, `empty-struct`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `two-fields` | `cell` | `[2, 1]` | `<cell [2 1]>` |
| `empty-struct` | `cell` | `[0, 1]` | `<cell [0 1]>` |

## Unresolved questions

- `fields.q-alias` — Confirm whether fields is exactly equivalent to fieldnames and whether it is deprecated.
