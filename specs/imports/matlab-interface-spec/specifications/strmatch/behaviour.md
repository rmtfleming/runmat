# `strmatch` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/strmatch_observations.json`](observations/strmatch_observations.json)

## Summary

Return the indices of the strings in an array that begin with a given prefix (or match exactly with the 'exact' option). (Documented as not recommended / legacy; verify by observation.)

## Confirmed invocation form

- `idx = strmatch(str, arr)` — exercised in the experiment (case `prefix`).

Additional forms are proposed but not yet exercised:
- `idx = strmatch(str, arr, 'exact')` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `prefix`, `exact`, `no-match`)*
2. **Confirmed** — Returns a column vector of matching indices; the default matches by prefix, the 'exact' option requires full equality, and no match yields a 0-by-1 empty. *(cases: `prefix`, `exact`, `no-match`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `prefix` | `double` | `[2, 1]` | `[1;2]` |
| `exact` | `double` | `[1, 1]` | `1` |
| `no-match` | `double` | `[0, 1]` | `zeros(0,1)` |

## Unresolved questions

- `strmatch.q-deprecated` — strmatch is documented as not recommended; record whether it warns and the suggested replacements.
- `strmatch.q-shape` — Confirm output shape/orientation of the index vector.
