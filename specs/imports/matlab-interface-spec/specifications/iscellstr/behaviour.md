# `iscellstr` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/iscellstr_observations.json`](observations/iscellstr_observations.json)

## Summary

Return a logical scalar that is true when the input is a cell array whose every element is a character vector.

## Confirmed invocation forms

- `tf = iscellstr(A)` — exercised in the experiment (case `all-char`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `all-char`, `mixed`, `char`, `empty-cell`)*
2. **Confirmed** — Returns a 1-by-1 logical: true when every element is a character vector (including the empty cell {}), false when any element is non-char or the input is a plain char array. *(cases: `all-char`, `empty-cell`, `mixed`, `char`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `all-char` | `logical` | `[1, 1]` | `true` |
| `mixed` | `logical` | `[1, 1]` | `false` |
| `char` | `logical` | `[1, 1]` | `false` |
| `empty-cell` | `logical` | `[1, 1]` | `true` |

## Unresolved questions

- `iscellstr.q-strings` — Behaviour for a cell array of string (not char) elements.
