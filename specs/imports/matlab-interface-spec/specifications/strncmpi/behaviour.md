# `strncmpi` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/strncmpi_observations.json`](observations/strncmpi_observations.json)

## Summary

Compare the first n characters of two character arrays, ignoring case, and return whether they match.

## Confirmed invocation forms

- `tf = strncmpi(s1, s2, n)` — exercised in the experiment (case `match`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `match`, `prefix-equal`, `no-match`)*
2. **Confirmed** — Returns a 1-by-1 logical that is true when the first n characters match ignoring case, and false when a character within the first n differs. *(cases: `match`, `prefix-equal`, `no-match`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `match` | `logical` | `[1, 1]` | `true` |
| `prefix-equal` | `logical` | `[1, 1]` | `true` |
| `no-match` | `logical` | `[1, 1]` | `false` |

## Unresolved questions

- `strncmpi.q-arrays` — Behaviour with cell-array inputs (element-wise).
