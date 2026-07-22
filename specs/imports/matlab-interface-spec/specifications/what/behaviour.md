# `what` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/what_observations.json`](observations/what_observations.json)

## Summary

Return a structure listing the MATLAB-related files in a folder.

## Confirmed invocation forms

- `s = what(folder)` — exercised in the experiment (case `tempdir`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `struct` for the observed inputs. *(cases: `tempdir`)*
2. **Confirmed** — Returns a struct describing the MATLAB-related files in the folder; the field names and contents are folder-dependent. *(cases: `tempdir`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `tempdir` | `struct` | `[1, 1]` | `<struct [1 1]>` |

## Unresolved questions

- `what.q-fields` — Confirm the field names of the returned structure.
- `what.q-content` — The listed files depend on the folder; record only the structure/type.
