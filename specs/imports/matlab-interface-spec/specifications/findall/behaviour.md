# `findall` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/findall_observations.json`](observations/findall_observations.json)

## Summary

Find graphics objects under a handle, optionally filtered by Type or by property; it found text objects that findobj did not.

## Confirmed invocation forms

- `h = findall(obj)` — All objects under obj, including obj. *(e.g. case `self`)*
- `h = findall(obj, 'Type', t)` — Objects of a given Type. *(e.g. case `type-line`)*
- `h = findall(obj, '-property', name)` — Objects that have a given property (COBRA form). *(e.g. case `property-fontsize`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — 'Type','line' on a figure with two lines returned a 2x1 Line array; 'Type','axes' returned the one Axes. *(cases: `type-line`, `type-axes`)*
2. **Confirmed** — On a figure with a title and an xlabel, findall(f, 'Type', 'text') found 2 objects where findobj found 0 (which objects they were was not recorded). *(cases: `type-text-hidden`)*
3. **Confirmed** — '-property','FontSize' on a figure with lines, title, label and a text object returned objects of Type axes and text only. *(cases: `property-fontsize`)*
4. **Confirmed** — set(findall(fig, '-property', 'FontSize'), 'FontSize', 12) set FontSize 12 on the axes and on a text object. *(cases: `set-fontsize`)*
5. **Confirmed** — findall(fig) includes fig itself; the first element had Type 'figure'. *(cases: `self`)*
6. **Confirmed** — No match returns a 0x0 matlab.graphics.GraphicsPlaceholder array. *(cases: `no-match`)*
7. **Confirmed** — A deleted figure handle returns the same 0x0 GraphicsPlaceholder array without error. *(cases: `deleted`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The first argument is a number that is not a graphics handle. | `MATLAB:findall:InvalidHandles` | `err-not-handle` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `type-line` | `struct` | `[1, 1]` | numel=2; size=[2 1]; class='matlab.graphics.chart.primitive.Line' |
| `type-axes` | `struct` | `[1, 1]` | numel=1; size=[1 1]; class='matlab.graphics.axis.Axes' |
| `type-text-hidden` | `struct` | `[1, 1]` | findall=2; findobj=0 |
| `property-fontsize` | `cell` | `[1, 2]` | ["'axes'", "'text'"] |
| `set-fontsize` | `struct` | `[1, 1]` | axes_fontsize=12; text_fontsize=12 |
| `no-match` | `struct` | `[1, 1]` | class='matlab.graphics.GraphicsPlaceholder'; size=[0 0] |
| `self` | `struct` | `[1, 1]` | contains_figure=true; first_type='figure' |
| `err-not-handle` | — | — | error `MATLAB:findall:InvalidHandles` |
| `deleted` | `matlab.graphics.GraphicsPlaceholder` | `[0, 0]` | <matlab.graphics.GraphicsPlaceholder [0 0]> |

## Unresolved questions

- `findall.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `findall.q-order` — The order of the returned objects beyond the first element was not recorded.
