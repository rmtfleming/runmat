# `isstr` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isstr_observations.json`](observations/isstr_observations.json)

## Summary

Return a logical scalar that is true when the input is a character array (a legacy equivalent of ischar).

## Confirmed invocation forms

- `tf = isstr(A)` — exercised in the experiment (case `char`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `char`, `numeric`, `string`, `cell`)*
2. **Confirmed** — Returns a 1-by-1 logical: true for a character array, false for numeric, a string scalar, and a cell input (a double-quoted string is not treated as char). *(cases: `char`, `numeric`, `string`, `cell`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `char` | `logical` | `[1, 1]` | `true` |
| `numeric` | `logical` | `[1, 1]` | `false` |
| `string` | `logical` | `[1, 1]` | `false` |
| `cell` | `logical` | `[1, 1]` | `false` |

## Unresolved questions

- `isstr.q-deprecated` — Confirm equivalence to ischar and any deprecation notice.
