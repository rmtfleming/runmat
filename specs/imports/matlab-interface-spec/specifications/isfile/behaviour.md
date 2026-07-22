# `isfile` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isfile_observations.json`](observations/isfile_observations.json)

## Summary

Return a logical scalar that is true when the given path refers to an existing file and false otherwise.

## Confirmed invocation form

- `tf = isfile(path)` — exercised in the experiment (case `missing`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `missing`)*
2. **Confirmed** — Returns a 1-by-1 logical; a path that does not exist yields false. *(cases: `missing`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `missing` | `logical` | `[1, 1]` | `false` |

## Unresolved questions

- `isfile.q-existing` — Add a case that creates a temporary file, tests isfile=true, then deletes it (self-cleaning).
- `isfile.q-folder` — Behaviour when the path is a folder rather than a file.
