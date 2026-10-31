# `allchild` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/allchild_observations.json`](observations/allchild_observations.json)

## Summary

Return the children of graphics objects; for the observed figure it returned more objects than its Children property lists; for an array of handles, a cell array.

## Confirmed invocation forms

- `h = allchild(obj)` — Children of one object (COBRA form allchild(0) for the root). *(e.g. case `root-has-figure`)*
- `c = allchild([obj1 obj2])` — Cell array with one entry per object. *(e.g. case `array`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — allchild(0) returned a column whose elements include a newly created invisible figure (class matlab.ui.Figure observed). *(cases: `root-has-figure`)*
2. **Confirmed** — For a figure with one axes, allchild returned 2 objects, of Type 'annotationpane' and 'axes', where fig.Children held 1. *(cases: `figure`)*
3. **Confirmed** — For axes with one line and a title, allchild returned 1 object, of Type 'line', the same count as the Children property. *(cases: `axes-vs-children`)*
4. **Confirmed** — For a line, the result is a 0x0 GraphicsPlaceholder array. *(cases: `leaf`)*
5. **Confirmed** — For [fig1 fig2] the result is a 2x1 cell (both elements of class matlab.graphics.primitive.world.Group in the observed run). *(cases: `array`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The input is a deleted handle, a number that is not a handle, or char. | `MATLAB:allchild:InvalidHandles` | `err-deleted`, `err-not-handle`, `err-char` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `root-has-figure` | `struct` | `[1, 1]` | contains_figure=true; class='matlab.ui.Figure'; is_column=true |
| `axes-vs-children` | `struct` | `[1, 1]` | allchild=1; children=1; types=["'line'"] |
| `leaf` | `struct` | `[1, 1]` | class='matlab.graphics.GraphicsPlaceholder'; size=[0 0] |
| `array` | `struct` | `[1, 1]` | class='cell'; size=[2 1]; element_classes=["'matlab.graphics.primitive.world.Group'", "'matlab.graphics.primitive.world.Group'"] |
| `figure` | `struct` | `[1, 1]` | allchild=2; children=1; types=["'annotationpane'", "'axes'"] |
| `err-deleted` | — | — | error `MATLAB:allchild:InvalidHandles` |
| `err-not-handle` | — | — | error `MATLAB:allchild:InvalidHandles` |
| `err-char` | — | — | error `MATLAB:allchild:InvalidHandles` |

## Unresolved questions

- `allchild.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `allchild.q-array-elements` — For an array input, the element class (world.Group arrays) was recorded but their contents were not.
- `allchild.q-order` — The order of children was not recorded.
