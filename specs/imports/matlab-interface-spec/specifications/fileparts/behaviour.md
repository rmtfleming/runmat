# `fileparts` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/fileparts_observations.json`](observations/fileparts_observations.json)

## Summary

Split a file path into its directory, file name, and extension parts.

## Confirmed invocation form

- `[filepath, name, ext] = fileparts(filename)` — exercised in the experiment (case `full-path`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `char` for the observed inputs. *(cases: `full-path`, `name-only`, `trailing-sep`, `no-ext`)*
2. **Confirmed** — The first output is the directory part: it is empty for a bare file name, and a trailing separator is removed. *(cases: `full-path`, `name-only`, `trailing-sep`, `no-ext`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `full-path` | `char` | `[1, 4]` | `'/a/b'` |
| `name-only` | `char` | `[0, 0]` | `''` |
| `trailing-sep` | `char` | `[1, 4]` | `'/x/y'` |
| `no-ext` | `char` | `[0, 0]` | `''` |

## Unresolved questions

- `fileparts.q-outputs` — Specify the second and third outputs (name, ext); the harness records only the first output (filepath).
- `fileparts.q-platform` — Record separator handling and platform dependence.
