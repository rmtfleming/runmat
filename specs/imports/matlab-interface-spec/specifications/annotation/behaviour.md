# `annotation` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/annotation_observations.json`](observations/annotation_observations.json)

## Summary

Create an annotation object (such as a line or arrow) on the current figure and return its handle.

## Confirmed invocation forms

- `an = annotation(type, x, y)` — exercised in the experiment (case `handle`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — annotation('line', x, y) returns an annotation graphics handle (observed class matlab.graphics.shape.Line; version-specific). *(cases: `handle`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `handle` | `matlab.graphics.shape.Line` | `[1, 1]` | `<matlab.graphics.shape.Line [1 1]>` |

## Unresolved questions

- `annotation.q-types` — Other annotation types (arrow, textbox, ...) are not exercised.
