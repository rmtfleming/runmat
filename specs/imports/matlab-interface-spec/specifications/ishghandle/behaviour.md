# `ishghandle` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/ishghandle_observations.json`](observations/ishghandle_observations.json)

## Summary

Return, elementwise, whether values are valid graphics handles, optionally of a given type.

## Confirmed invocation forms

- `tf = ishghandle(h)` — True for valid graphics handles. *(e.g. case `figure`)*
- `tf = ishghandle(h, type)` — True for valid handles of the given Type (COBRA form with 'figure'). *(e.g. case `figure-type`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — True for live figure, axes and line handles, for an array [f ax ln] (elementwise), for double(f) of a live figure, and for the value 0 and groot. *(cases: `figure`, `line`, `array`, `double-of-figure`, `root`, `groot`)*
2. **Confirmed** — ishghandle(f, 'figure') and ishghandle(ax, 'axes') are true; ishghandle(ax, 'figure') is false. *(cases: `figure-type`, `axes-type`, `axes-as-figure`)*
3. **Confirmed** — False for a deleted figure, a number that is not a handle, char, struct and a graph object; the observed non-integer numeric matrix gave a false matrix of its size. *(cases: `deleted`, `number`, `char`, `struct`, `graph`, `matrix-number`)*
4. **Confirmed** — Empty input gives a 0x0 logical. *(cases: `empty`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| No input is given. | `MATLAB:minrhs` | `err-none` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `figure` | `logical` | `[1, 1]` | true |
| `figure-type` | `logical` | `[1, 1]` | true |
| `axes-as-figure` | `logical` | `[1, 1]` | false |
| `axes-type` | `logical` | `[1, 1]` | true |
| `line` | `logical` | `[1, 1]` | true |
| `array` | `logical` | `[1, 3]` | [true true true] |
| `deleted` | `logical` | `[1, 1]` | false |
| `double-of-figure` | `logical` | `[1, 1]` | true |
| `root` | `logical` | `[1, 1]` | true |
| `groot` | `logical` | `[1, 1]` | true |
| `number` | `logical` | `[1, 1]` | false |
| `empty` | `logical` | `[0, 0]` | [] |
| `matrix-number` | `logical` | `[2, 2]` | [false false;false false] |
| `char` | `logical` | `[1, 1]` | false |
| `struct` | `logical` | `[1, 1]` | false |
| `graph` | `logical` | `[1, 1]` | false |
| `err-none` | — | — | error `MATLAB:minrhs` |

## Unresolved questions

- `ishghandle.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `ishghandle.q-numeric-handles` — Which numeric values count as handles (integer figure numbers, other integers) is not established beyond double(f) of a live figure, 0, and the non-integer values tried.
