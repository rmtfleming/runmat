# `fplot` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N5, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/fplot_observations.json`](observations/fplot_observations.json)

## Summary

Plot a function of x over an interval (default [-5 5]) as a FunctionLine object, applying line properties given as a LineSpec or name-value pairs.

## Confirmed invocation forms

- `fplot(f, Name, Value, ...)` — Plot with line properties, no output (COBRA form). *(e.g. case `cobra-no-output`)*
- `h = fplot(f)` — Return the line object. *(e.g. case `default`)*
- `h = fplot(f, [xmin xmax])` — Plot over an interval. *(e.g. case `interval`)*
- `h = fplot(f, LineSpec)` — Line style and colour as a LineSpec. *(e.g. case `linespec`)*
- `h = fplot(ax, f, ...)` — Draw into the given axes. *(e.g. case `ax-arg`)*

## Name-value arguments

- `LineWidth` — Line width (1.5 observed).
- `Color` — Line colour ('r' observed, stored as [1 0 0]).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The returned object is a matlab.graphics.function.FunctionLine whose parent is the current axes, plotted at more than two points. *(cases: `default`)*
2. **Confirmed** — Without an interval, XData spans [-5, 5] and the x-axis limits are [-5 5]. *(cases: `default`, `xlim-default`)*
3. **Confirmed** — With [0 2], XData spans [0, 2], and YData equals f(XData) exactly for @(x) x.^2. *(cases: `interval`, `values`)*
4. **Confirmed** — For the COBRA call fplot(@(x) x, 'LineWidth', 1.5, 'Color', 'r'), the returned FunctionLine has LineWidth 1.5 and Color [1 0 0] and its parent is the current axes; without an output, the axes contain one FunctionLine with those values. *(cases: `cobra-nv`, `cobra-no-output`)*
5. **Confirmed** — LineSpec 'r--' gave Color [1 0 0] and LineStyle '--'; the default LineWidth is 0.5 and LineStyle '-'. *(cases: `linespec`, `default`)*
6. **Confirmed** — With an axes first argument, the line is drawn into that axes. *(cases: `ax-arg`)*

## Edge cases (observed)

- *(non-normative)* A numeric first argument (5) did not error; it was plotted over [-5 5] like a function. *(cases: `numeric-constant`)*
- *(non-normative)* Observed: the default Color on R2026a was [0.066 0.443 0.745]. *(cases: `default`)*
- *(non-normative)* Inference (not observed): the default Color may depend on the graphics theme and release. *(cases: `default`)*

## Warnings (confirmed)

| condition | identifier | case |
|---|---|---|
| f is a char expression ('sin(x)'); the plot is still drawn. | `MATLAB:fplot:StringFunctionsDeprecated` | `char-fn` |

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The interval is decreasing ([2 1]). | `MATLAB:fplot:RangeVector` | `err-decreasing` |
| A name-value name is not a property. | `MATLAB:fplot:InvalidParameter` | `err-bad-property` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `default` | `struct` | `[1, 1]` | warning_id=''; class='matlab.graphics.function.FunctionLine'; x_min=-5; x_max=5; more_than_two_points=true; LineWidth=0.5; Color=[0.066000000000000003 0.443 0.745]; LineStyle='-'; parent_is_gca=true |
| `cobra-nv` | `struct` | `[1, 1]` | warning_id=''; class='matlab.graphics.function.FunctionLine'; x_min=-5; x_max=5; more_than_two_points=true; LineWidth=1.5; Color=[1 0 0]; LineStyle='-'; parent_is_gca=true |
| `cobra-no-output` | `struct` | `[1, 1]` | numel=1; class='matlab.graphics.function.FunctionLine'; LineWidth=1.5; Color=[1 0 0] |
| `interval` | `struct` | `[1, 1]` | warning_id=''; class='matlab.graphics.function.FunctionLine'; x_min=0; x_max=2; more_than_two_points=true; LineWidth=0.5; Color=[0.066000000000000003 0.443 0.745]; LineStyle='-'; parent_is_gca=true |
| `values` | `double` | `[1, 1]` | 0 |
| `linespec` | `struct` | `[1, 1]` | warning_id=''; class='matlab.graphics.function.FunctionLine'; x_min=-5; x_max=5; more_than_two_points=true; LineWidth=0.5; Color=[1 0 0]; LineStyle='--'; parent_is_gca=true |
| `xlim-default` | `double` | `[1, 2]` | [-5 5] |
| `ax-arg` | `logical` | `[1, 1]` | true |
| `char-fn` | `struct` | `[1, 1]` | warning_id='MATLAB:fplot:StringFunctionsDeprecated'; class='matlab.graphics.function.FunctionLine'; x_min=-5; x_max=5; more_than_two_points=true; LineWidth=0.5; Color=[0.066000000000000003 0.443 0.745]; LineStyle='-'; parent_is_gca=true |
| `numeric-constant` | `struct` | `[1, 1]` | warning_id=''; class='matlab.graphics.function.FunctionLine'; x_min=-5; x_max=5; more_than_two_points=true; LineWidth=0.5; Color=[0.066000000000000003 0.443 0.745]; LineStyle='-'; parent_is_gca=true |
| `err-decreasing` | — | — | error `MATLAB:fplot:RangeVector` |
| `err-bad-property` | — | — | error `MATLAB:fplot:InvalidParameter` |

## Unresolved questions

- `fplot.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `fplot.q-theme` — The graphics theme active during the run was not recorded; default colours and the default LineWidth may depend on it and on the release (constitution 8.2).
- `fplot.q-points` — The number and placement of evaluation points were not recorded and are not specified.
