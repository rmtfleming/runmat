# `writetable` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/writetable_observations.json`](observations/writetable_observations.json)

## Summary

Write the contents of a table to a file (for example a delimited text or spreadsheet file).

## Confirmed invocation form

- `writetable(T, filename)` — exercised in the experiment (case `file-created`).

Additional forms are proposed but not yet exercised:
- `writetable(T)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Writing a table to a file and reading it back reproduces the data and shape (a 2-by-2 numeric table round-tripped to [1 2;3 4], size [2 2]); after the call the target file exists (isfile is true). writetable returns no output; the effect is the written file. *(cases: `file-created`, `roundtrip-size`, `roundtrip-values`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `file-created` | `logical` | `[1, 1]` | `true` |
| `roundtrip-size` | `double` | `[1, 2]` | `[2 2]` |
| `roundtrip-values` | `double` | `[2, 2]` | `[1 2;3 4]` |

## Unresolved questions

- `writetable.q-side-effect` — writetable writes a file; experiments must target a temporary path, verify by reading back, then delete it.
- `writetable.q-formats` — Enumerate supported file formats and default naming.
