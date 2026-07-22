# `saveas` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/saveas_observations.json`](observations/saveas_observations.json)

## Summary

Save a figure or graphics object to a file in a specified format.

## Confirmed invocation form

- `saveas(fig, filename)` — exercised in the experiment (case `file-created`).

Additional forms are proposed but not yet exercised:
- `saveas(fig, filename, formattype)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Saving a graphics object creates the target file (isfile is true) with a nonzero byte size (an off-screen line plot produced a PNG of about 12 kB). saveas returns no output; the effect is the written file. *(cases: `file-created`, `file-nonempty`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `file-created` | `logical` | `[1, 1]` | `true` |
| `file-nonempty` | `double` | `[1, 1]` | `12312` |

## Unresolved questions

- `saveas.q-side-effect` — saveas requires a graphics object and writes a file; experiments must create a figure, target a temporary path, verify, and clean up.
- `saveas.q-formats` — Enumerate supported format types and extension inference.
