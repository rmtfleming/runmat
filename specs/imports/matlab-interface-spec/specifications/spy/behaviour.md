# `spy` — behaviour specification

- **Spec version:** 0.3.0
- **Status:** approved 2026-10-29 (Tier B widening W2, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/spy_observations.json`](observations/spy_observations.json)

## Summary

Plot the sparsity pattern of a matrix as markers at the nonzero positions; returns no output.

## Confirmed invocation forms

- `spy(A)` — Plot nonzero positions of A. *(e.g. case `points`)*
- `spy(A, LineSpec)` — Plot with a marker and colour specification. *(e.g. case `linespec`)*
- `spy(A, markersize)` — Plot with a numeric marker size. *(e.g. case `markersize`)*
- `spy(A, LineSpec, markersize)` — Both. *(e.g. case `linespec-markersize`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — spy(A) draws into the current axes and returns no output value; requesting an output errors. *(cases: `plots`, `output-arg`)*
2. **Confirmed** — The axes receives exactly one Line object with markers and no connecting line (LineStyle 'none'). *(cases: `points`, `dense`, `all-zero`)*
3. **Confirmed** — The Line's XData are the column indices and YData the row indices of the nonzeros, in column-major order: [1 0 1; 0 1 0] gave XData [1 2 3], YData [1 2 1]. *(cases: `points`, `dense`, `tall`, `logical`)*
4. **Confirmed** — The y axis is reversed (YDir 'reverse'); for an m-by-n matrix XLim is [0 n+1] and YLim is [0 m+1]; x and y data units are equal (the first two DataAspectRatio entries are equal). *(cases: `points`, `tall`, `all-zero`)*
5. **Confirmed** — The x-axis label is 'nz = ' followed by the number of nonzeros (3, 2, and 0 for an all-zero matrix). *(cases: `points`, `dense`, `all-zero`)*
6. **Confirmed** — With no style arguments the marker is '.'. *(cases: `default-style`, `points`)*
7. **Confirmed** — A LineSpec sets marker and colour: 'r*' gave marker '*' and colour [1 0 0]; 'g+' gave '+' and [0 1 0]. *(cases: `linespec`, `linespec-markersize`)*
8. **Confirmed** — A numeric marker size sets MarkerSize (20 gave 20; with 'g+', 12 gave 12). *(cases: `markersize`, `linespec-markersize`)*
9. **Confirmed** — An all-zero matrix still produces one Line, with NaN coordinates. *(cases: `all-zero`)*
10. **Confirmed** — Full numeric, logical and char inputs are accepted; nonzero elements are plotted. *(cases: `dense`, `logical`, `char`)*

## Edge cases (observed)

- *(non-normative)* Observed for small matrices only: default MarkerSize 14 (6 when a LineSpec was given without a size) and default colour [0.066 0.443 0.745]. *(cases: `default-style`, `linespec`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| An output argument is requested. | `MATLAB:TooManyOutputs` | `output-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `plots` | `logical` | `[1, 1]` | true |
| `points` | `struct` | `[1, 1]` | n_children=1; warning_id=''; child_classes=["'matlab.graphics.chart.primitive.Line'"]; XData=[1 2 3]; YData=[1 2 1]; Marker='.'; LineStyle='none'; Color=[0.066000000000000003 0.443 0.745]; MarkerSize=14; XLim=[0 4]; YLim=[0 3]; YDir='reverse'; XLabel='nz = 3'; DataAspectRatio=[0.5 0.5 1] |
| `dense` | `struct` | `[1, 1]` | n_children=1; warning_id=''; child_classes=["'matlab.graphics.chart.primitive.Line'"]; XData=[1 2]; YData=[2 1]; Marker='.'; LineStyle='none'; Color=[0.066000000000000003 0.443 0.745]; MarkerSize=14; XLim=[0 3]; YLim=[0 3]; YDir='reverse'; XLabel='nz = 2'; DataAspectRatio=[0.5 0.5 1] |
| `all-zero` | `struct` | `[1, 1]` | n_children=1; warning_id=''; child_classes=["'matlab.graphics.chart.primitive.Line'"]; XData=NaN; YData=NaN; Marker='.'; LineStyle='none'; Color=[0.066000000000000003 0.443 0.745]; MarkerSize=14; XLim=[0 4]; YLim=[0 4]; YDir='reverse'; XLabel='nz = 0'; DataAspectRatio=[1 1 1] |
| `tall` | `struct` | `[1, 1]` | n_children=1; warning_id=''; child_classes=["'matlab.graphics.chart.primitive.Line'"]; XData=[1 1]; YData=[1 4]; Marker='.'; LineStyle='none'; Color=[0.066000000000000003 0.443 0.745]; MarkerSize=14; XLim=[0 2]; YLim=[0 5]; YDir='reverse'; XLabel='nz = 2'; DataAspectRatio=[0.5 0.5 1] |
| `linespec` | `struct` | `[1, 1]` | n_children=1; warning_id=''; child_classes=["'matlab.graphics.chart.primitive.Line'"]; XData=[1 2]; YData=[1 2]; Marker='*'; LineStyle='none'; Color=[1 0 0]; MarkerSize=6; XLim=[0 3]; YLim=[0 3]; YDir='reverse'; XLabel='nz = 2'; DataAspectRatio=[0.5 0.5 1] |
| `markersize` | `struct` | `[1, 1]` | n_children=1; warning_id=''; child_classes=["'matlab.graphics.chart.primitive.Line'"]; XData=[1 2]; YData=[1 2]; Marker='.'; LineStyle='none'; Color=[0.066000000000000003 0.443 0.745]; MarkerSize=20; XLim=[0 3]; YLim=[0 3]; YDir='reverse'; XLabel='nz = 2'; DataAspectRatio=[0.5 0.5 1] |
| `linespec-markersize` | `struct` | `[1, 1]` | n_children=1; warning_id=''; child_classes=["'matlab.graphics.chart.primitive.Line'"]; XData=[1 2]; YData=[1 2]; Marker='+'; LineStyle='none'; Color=[0 1 0]; MarkerSize=12; XLim=[0 3]; YLim=[0 3]; YDir='reverse'; XLabel='nz = 2'; DataAspectRatio=[0.5 0.5 1] |
| `default-style` | `struct` | `[1, 1]` | n_children=1; warning_id=''; child_classes=["'matlab.graphics.chart.primitive.Line'"]; XData=[1 2]; YData=[1 2]; Marker='.'; LineStyle='none'; Color=[0.066000000000000003 0.443 0.745]; MarkerSize=14; XLim=[0 3]; YLim=[0 3]; YDir='reverse'; XLabel='nz = 2'; DataAspectRatio=[0.5 0.5 1] |
| `output-arg` | — | — | error `MATLAB:TooManyOutputs` |
| `logical` | `struct` | `[1, 1]` | n_children=1; warning_id=''; child_classes=["'matlab.graphics.chart.primitive.Line'"]; XData=[1 1 2]; YData=[1 2 2]; Marker='.'; LineStyle='none'; Color=[0.066000000000000003 0.443 0.745]; MarkerSize=14; XLim=[0 3]; YLim=[0 3]; YDir='reverse'; XLabel='nz = 3'; DataAspectRatio=[0.5 0.5 1] |
| `char` | `struct` | `[1, 1]` | n_children=1; warning_id=''; child_classes=["'matlab.graphics.chart.primitive.Line'"]; XData=[1 2]; YData=[1 1]; Marker='.'; LineStyle='none'; Color=[0.066000000000000003 0.443 0.745]; MarkerSize=14; XLim=[0 3]; YLim=[0 2]; YDir='reverse'; XLabel='nz = 2'; DataAspectRatio=[0.5 0.5 1] |

## Unresolved questions

- `spy.q-default-markersize` — Whether the default marker size depends on matrix size was not exercised (only matrices up to 4 rows/columns).
- `spy.q-large` — Behaviour for large matrices and complex input was not exercised.

Resolved in 0.3.0: `spy.q-markersize`.
