# `isdir` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isdir_observations.json`](observations/isdir_observations.json)

## Summary

Return a logical scalar that is true when the given path refers to an existing folder (a legacy equivalent of isfolder).

## Confirmed invocation forms

- `tf = isdir(path)` — exercised in the experiment (case `existing`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `existing`, `missing`)*
2. **Confirmed** — Returns a 1-by-1 logical: true for an existing folder path, false for a non-existent path. *(cases: `existing`, `missing`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `existing` | `logical` | `[1, 1]` | `true` |
| `missing` | `logical` | `[1, 1]` | `false` |

## Unresolved questions

- `isdir.q-deprecated` — Confirm equivalence to isfolder and any deprecation notice.
