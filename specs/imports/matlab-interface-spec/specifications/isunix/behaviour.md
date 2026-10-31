# `isunix` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isunix_observations.json`](observations/isunix_observations.json)

## Summary

Return a logical scalar; it was true on the GLNXA64 (Linux) oracle, and other platforms were not observed.

## Confirmed invocation forms

- `tf = isunix` — Platform test. *(e.g. case `value`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — On the GLNXA64 oracle isunix returned logical true, while ispc and ismac returned false. *(cases: `value`, `consistency`)*
2. **Confirmed** — The COBRA test `isunix && ~ismac` was true on the GLNXA64 oracle. *(cases: `cobra-linux-test`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| An input argument is given. | `MATLAB:maxrhs` | `err-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `value` | `logical` | `[1, 1]` | true |
| `consistency` | `struct` | `[1, 1]` | isunix=true; ispc=false; ismac=false |
| `cobra-linux-test` | `logical` | `[1, 1]` | true |
| `err-arg` | — | — | error `MATLAB:maxrhs` |

## Unresolved questions

- `isunix.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `isunix.q-platform` — Only the GLNXA64 (Linux) oracle was exercised; results on other platforms were not observed.
