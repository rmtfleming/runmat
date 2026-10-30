# `tiledlayout` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/tiledlayout_observations.json`](observations/tiledlayout_observations.json)

## Summary

Create a tiled chart layout for arranging axes in a grid in the current (or given) figure.

## Confirmed invocation forms

- `t = tiledlayout(m, n)` — Fixed m-by-n grid. *(e.g. case `grid`)*
- `t = tiledlayout('flow')` — Grid that grows as tiles are added. *(e.g. case `flow`)*
- `t = tiledlayout(___, Name, Value)` — With spacing options. *(e.g. case `spacing`)*
- `t = tiledlayout(parent, m, n)` — In a given figure. *(e.g. case `parent`)*

## Name-value arguments

- `TileSpacing` — 'loose' (default observed) and 'compact' observed; 'nope' errored.
- `Padding` — 'loose' (default observed), 'tight' observed.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is a matlab.graphics.layout.TiledChartLayout. *(cases: `grid`, `flow`, `spacing`)*
2. **Confirmed** — tiledlayout(2,3) gives GridSize [2 3], TileArrangement 'fixed', and TileSpacing and Padding 'loose', with no children until tiles are added. *(cases: `grid`)*
3. **Confirmed** — Each nexttile adds one child axes (two calls gave 2 children). *(cases: `tiles`, `flow-tiles`)*
4. **Confirmed** — 'flow' gives TileArrangement 'flow' and GridSize [1 1] when empty; after three tiles the GridSize was [2 2]. *(cases: `flow`, `flow-tiles`)*
5. **Confirmed** — TileSpacing 'compact' and Padding 'tight' are stored as given. *(cases: `spacing`)*
6. **Confirmed** — A figure passed first becomes the layout's Parent. *(cases: `parent`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A single numeric argument is given. | `MATLAB:narginchk:notEnoughInputs` | `err-one-arg` |
| A grid dimension is zero. | `MATLAB:tiledchartlayout:InvalidGridSize` | `err-zero` |
| TileSpacing is not a recognised value. | `MATLAB:datatypes:InvalidEnumValue` | `err-spacing` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `grid` | `struct` | `[1, 1]` | class='matlab.graphics.layout.TiledChartLayout'; GridSize=[2 3]; TileSpacing='loose'; Padding='loose'; TileArrangement='fixed'; n_children=0 |
| `tiles` | `struct` | `[1, 1]` | class='matlab.graphics.layout.TiledChartLayout'; GridSize=[2 2]; TileSpacing='loose'; Padding='loose'; TileArrangement='fixed'; n_children=2 |
| `flow` | `struct` | `[1, 1]` | class='matlab.graphics.layout.TiledChartLayout'; GridSize=[1 1]; TileSpacing='loose'; Padding='loose'; TileArrangement='flow'; n_children=0 |
| `flow-tiles` | `struct` | `[1, 1]` | class='matlab.graphics.layout.TiledChartLayout'; GridSize=[2 2]; TileSpacing='loose'; Padding='loose'; TileArrangement='flow'; n_children=3 |
| `spacing` | `struct` | `[1, 1]` | class='matlab.graphics.layout.TiledChartLayout'; GridSize=[1 2]; TileSpacing='compact'; Padding='tight'; TileArrangement='fixed'; n_children=0 |
| `err-one-arg` | — | — | error `MATLAB:narginchk:notEnoughInputs` |
| `parent` | `char` | `[1, 16]` | 'matlab.ui.Figure' |
| `err-zero` | — | — | error `MATLAB:tiledchartlayout:InvalidGridSize` |
| `err-spacing` | — | — | error `MATLAB:datatypes:InvalidEnumValue` |

## Unresolved questions

- `tiledlayout.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `tiledlayout.q-flow-growth` — The flow grid size was observed for 0 and 3 tiles only.
