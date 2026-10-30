# `isKey` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isKey_observations.json`](observations/isKey_observations.json)

## Summary

Return whether keys are present in a containers.Map.

## Confirmed invocation forms

- `tf = isKey(M, key)` — True if `key` is a key of M; for a cell array of keys, an array of results. *(e.g. case `present`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is logical. *(cases: `present`, `cell-query`)*
2. **Confirmed** — A present key gives true and an absent key false; matching is case-sensitive ('A' in a map with 'a' gave false). *(cases: `present`, `absent`, `case`)*
3. **Confirmed** — A cell array of keys gives a logical array of the same size (2x2 cell gave [true false; true true]). *(cases: `cell-query`, `numeric-cell`)*
4. **Confirmed** — Numeric keys work the same way. *(cases: `numeric`, `numeric-cell`)*
5. **Confirmed** — A char key on a numeric-keyed map, and the key 'a' on an empty map, gave false without error. *(cases: `type-mismatch`, `empty-map`)*
6. **Confirmed** — A string scalar key matches the equal char key. *(cases: `string-key`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `present` | `logical` | `[1, 1]` | true |
| `absent` | `logical` | `[1, 1]` | false |
| `case` | `logical` | `[1, 1]` | false |
| `cell-query` | `logical` | `[2, 2]` | [true false;true true] |
| `numeric` | `logical` | `[1, 2]` | [true false] |
| `numeric-cell` | `logical` | `[1, 2]` | [true false] |
| `type-mismatch` | `logical` | `[1, 1]` | false |
| `empty-map` | `logical` | `[1, 1]` | false |
| `string-key` | `logical` | `[1, 1]` | true |

## Unresolved questions

- `isKey.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
