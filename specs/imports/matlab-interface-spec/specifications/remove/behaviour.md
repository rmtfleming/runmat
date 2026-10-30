# `remove` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/remove_observations.json`](observations/remove_observations.json)

## Summary

Remove keys and their values from a containers.Map; the map object itself is modified and also returned.

## Confirmed invocation forms

- `M = remove(M, keySet)` — Remove one key or a cell array of keys. *(e.g. case `one`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The given key(s) are removed (one key, a cell array of keys, a numeric key, or every key). *(cases: `one`, `several`, `numeric`, `all`)*
2. **Confirmed** — The original map variable showed the same keys and Count as the returned map after the call. *(cases: `one`, `several`, `numeric`, `all`)*
3. **Confirmed** — The returned value is a containers.Map. *(cases: `one`, `absent`)*
4. **Confirmed** — With a cell list that includes an absent key, the present keys are still removed. *(cases: `partly-absent`)*

## Warnings (confirmed)

| condition | identifier | case |
|---|---|---|
| A key to remove is not in the map (also a char key on a numeric map); the call completes and the map keeps its other keys. | `MATLAB:Containers:Map:NoKeyToRemove` | `absent`, `partly-absent`, `type-mismatch` |

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `one` | `struct` | `[1, 1]` | keys_returned=["'b'", "'c'"]; keys_original=["'b'", "'c'"]; count_original=2; class='containers.Map'; warning_id='' |
| `several` | `struct` | `[1, 1]` | keys_returned=["'b'"]; keys_original=["'b'"]; count_original=1; class='containers.Map'; warning_id='' |
| `numeric` | `struct` | `[1, 1]` | keys_returned=["2", "3"]; keys_original=["2", "3"]; count_original=2; class='containers.Map'; warning_id='' |
| `all` | `struct` | `[1, 1]` | keys_returned=[]; keys_original=[]; count_original=0; class='containers.Map'; warning_id='' |
| `absent` | `struct` | `[1, 1]` | keys_returned=["'a'", "'b'", "'c'"]; keys_original=["'a'", "'b'", "'c'"]; count_original=3; class='containers.Map'; warning_id='MATLAB:Containers:Map:NoKeyToRemove' |
| `partly-absent` | `struct` | `[1, 1]` | keys_returned=["'b'", "'c'"]; keys_original=["'b'", "'c'"]; count_original=2; class='containers.Map'; warning_id='MATLAB:Containers:Map:NoKeyToRemove' |
| `type-mismatch` | `struct` | `[1, 1]` | keys_returned=["1", "2", "3"]; keys_original=["1", "2", "3"]; count_original=3; class='containers.Map'; warning_id='MATLAB:Containers:Map:NoKeyToRemove' |

## Unresolved questions

- `remove.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
