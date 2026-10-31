# `rectangle` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N5, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/rectangle_observations.json`](observations/rectangle_observations.json)

## Summary

Draw a rectangle in the current axes and return the Rectangle object; Position, Curvature and colour properties are stored as given.

## Confirmed invocation forms

- `rectangle(Name, Value, ...)` — Set properties such as Position, EdgeColor, LineWidth and LineStyle (COBRA form). *(e.g. case `cobra-form`)*
- `h = rectangle()` — Default rectangle. *(e.g. case `default`)*
- `h = rectangle(ax, Name, Value, ...)` — Draw into the given axes. *(e.g. case `ax-arg`)*

## Name-value arguments

- `Position` — [x y w h]; default [0 0 1 1]; a negative width and a 3-element value were rejected.
- `Curvature` — Scalar or [cx cy]; default [0 0]; a value above 1 was rejected.
- `FaceColor` — Fill colour; default 'none'.
- `EdgeColor` — Edge colour.
- `LineWidth` — Default 0.5.
- `LineStyle` — Default '-'.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The default object is a matlab.graphics.primitive.Rectangle with Position [0 0 1 1], Curvature [0 0], FaceColor 'none', LineWidth 0.5 and LineStyle '-'. *(cases: `default`)*
2. **Confirmed** — The COBRA call rectangle('Position', [0, 0, 40, 0.1], 'EdgeColor', 'black', 'LineWidth', 2, 'LineStyle', '-.') gives Position [0 0 40 0.1], EdgeColor [0 0 0], LineWidth 2 and LineStyle '-.', and in fresh axes the limits became x [0 40], y [0 0.1]. *(cases: `cobra-form`, `cobra-limits`)*
3. **Confirmed** — Curvature is stored as given: scalar 1 stays 1, [0.5 0.2] stays [0.5 0.2]. *(cases: `curvature-scalar`, `curvature-pair`)*
4. **Confirmed** — FaceColor 'r' is stored as [1 0 0]. *(cases: `face-color`)*
5. **Confirmed** — Without an output, one Rectangle is added to the current axes; with an axes first argument it is drawn into that axes. *(cases: `no-output`, `ax-arg`)*

## Edge cases (observed)

- *(non-normative)* Observed: the default EdgeColor on R2026a was [0.1294 0.1294 0.1294]. *(cases: `default`)*
- *(non-normative)* Inference (not observed): the default EdgeColor may depend on the graphics theme and release. *(cases: `default`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| Position has a negative width. | `MATLAB:hg:set_chck:DimensionsOutsideRange` | `err-negative-width` |
| Position does not have 4 elements. | `MATLAB:hg:dt_conv:Matrix_to_matlabRect:Expected4ElementVector` | `err-position-length` |
| A Curvature value is above 1. | `MATLAB:hg:shaped_arrays:CurvatureRange` | `err-curvature-range` |
| A name is not a property. | `MATLAB:hg:InvalidProperty` | `err-bad-property` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `default` | `struct` | `[1, 1]` | class='matlab.graphics.primitive.Rectangle'; Position=[0 0 1 1]; Curvature=[0 0]; FaceColor='none'; EdgeColor=[0.12941176470588237 0.12941176470588237 0.12941176470588237]; LineWidth=0.5; LineStyle='-' |
| `cobra-form` | `struct` | `[1, 1]` | class='matlab.graphics.primitive.Rectangle'; Position=[0 0 40 0.10000000000000001]; Curvature=[0 0]; FaceColor='none'; EdgeColor=[0 0 0]; LineWidth=2; LineStyle='-.' |
| `cobra-limits` | `struct` | `[1, 1]` | xlim=[0 40]; ylim=[0 0.10000000000000001] |
| `curvature-scalar` | `struct` | `[1, 1]` | class='matlab.graphics.primitive.Rectangle'; Position=[1 2 3 4]; Curvature=1; FaceColor='none'; EdgeColor=[0.12941176470588237 0.12941176470588237 0.12941176470588237]; LineWidth=0.5; LineStyle='-' |
| `curvature-pair` | `struct` | `[1, 1]` | class='matlab.graphics.primitive.Rectangle'; Position=[1 2 3 4]; Curvature=[0.5 0.20000000000000001]; FaceColor='none'; EdgeColor=[0.12941176470588237 0.12941176470588237 0.12941176470588237]; LineWidth=0.5; LineStyle='-' |
| `face-color` | `struct` | `[1, 1]` | class='matlab.graphics.primitive.Rectangle'; Position=[0 0 1 1]; Curvature=[0 0]; FaceColor=[1 0 0]; EdgeColor=[0.12941176470588237 0.12941176470588237 0.12941176470588237]; LineWidth=0.5; LineStyle='-' |
| `no-output` | `struct` | `[1, 1]` | numel=1; class='matlab.graphics.primitive.Rectangle'; Position=[1 1 2 2] |
| `ax-arg` | `logical` | `[1, 1]` | true |
| `err-negative-width` | — | — | error `MATLAB:hg:set_chck:DimensionsOutsideRange` |
| `err-position-length` | — | — | error `MATLAB:hg:dt_conv:Matrix_to_matlabRect:Expected4ElementVector` |
| `err-curvature-range` | — | — | error `MATLAB:hg:shaped_arrays:CurvatureRange` |
| `err-bad-property` | — | — | error `MATLAB:hg:InvalidProperty` |

## Unresolved questions

- `rectangle.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `rectangle.q-theme` — The graphics theme active during the run was not recorded; default colours and the default LineWidth may depend on it and on the release (constitution 8.2).
- `rectangle.q-error-ids` — The error identifiers come from the generic graphics property layer (MATLAB:hg:...); whether an implementation must reproduce them is a human-review question (constitution 8.2).
