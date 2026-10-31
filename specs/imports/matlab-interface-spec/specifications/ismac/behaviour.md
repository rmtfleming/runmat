# `ismac` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/ismac_observations.json`](observations/ismac_observations.json)

## Summary

Return a logical scalar; it was false on the GLNXA64 (Linux) oracle, and other platforms were not observed.

## Confirmed invocation forms

- `tf = ismac` — Platform test. *(e.g. case `value`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — On the GLNXA64 oracle ismac returned logical false. *(cases: `value`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| An input argument is given. | `MATLAB:maxrhs` | `err-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `value` | `logical` | `[1, 1]` | false |
| `err-arg` | — | — | error `MATLAB:maxrhs` |

## Unresolved questions

- `ismac.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `ismac.q-platform` — Only the GLNXA64 (Linux) oracle was exercised; results on other platforms were not observed.
