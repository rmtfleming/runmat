# `pcolor` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/pcolor_observations.json`](observations/pcolor_observations.json)

## Summary

Draw a flat-shaded surface with zero height whose colours come from C, and return the Surface object.

## Confirmed invocation forms

- `h = pcolor(C)` — Grid coordinates 1..n and 1..m. *(e.g. case `c-only`)*
- `h = pcolor(X, Y, C)` — Given coordinates (vectors, as in COBRA, or matrices). *(e.g. case `xyc-vectors`)*
- `h = pcolor(ax, C)` — Draw into the given axes. *(e.g. case `ax-arg`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is a matlab.graphics.primitive.Surface. *(cases: `c-only`, `xyc-vectors`, `xyc-matrices`, `no-ax-arg`, `ax-arg`)*
2. **Confirmed** — pcolor(ax, C) drew into ax (h.Parent == ax) with CData equal to C. *(cases: `ax-arg`)*
3. **Confirmed** — pcolor(C) for a 2x3 C gave XData [1 2 3], YData [1;2], ZData zeros(2,3), CData equal to C. *(cases: `c-only`)*
4. **Confirmed** — With vectors x, y the XData/YData are the vectors as given (row vectors stayed rows); with meshgrid matrices they are the matrices; ZData is zeros the size of C and CData is C. *(cases: `xyc-vectors`, `xyc-matrices`)*
5. **Confirmed** — Without an axes argument the surface is drawn into the current axes (h.Parent == gca). *(cases: `no-ax-arg`)*
6. **Confirmed** — After pcolor the axes view was [0 90] (seen from above). *(cases: `view`)*
7. **Confirmed** — FaceColor is 'flat'. *(cases: `edge-face`)*

## Edge cases (observed)

- *(non-normative)* Observed with default graphics settings: EdgeColor was [0.12941176470588237 0.12941176470588237 0.12941176470588237]; not required, as it may follow default settings that were not exercised. *(cases: `edge-face`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| x or y length does not match the columns or rows of C. | `MATLAB:pcolor:InputSizeMismatch` | `err-size` |
| Two numeric arguments are given: a vector then a matrix (pcolor([1 2], [1 2; 3 4]) observed). | `MATLAB:badsubscript` | `err-two-args` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `c-only` | `struct` | `[1, 1]` | class='matlab.graphics.primitive.Surface'; XData=[1 2 3]; YData=[1;2]; ZData=[0 0 0;0 0 0]; CData=[1 2 3;4 5 6] |
| `ax-arg` | `struct` | `[1, 1]` | class='matlab.graphics.primitive.Surface'; parent_is_ax=true; CData=[1 2 3;4 5 6] |
| `xyc-vectors` | `struct` | `[1, 1]` | class='matlab.graphics.primitive.Surface'; XData=[10 20 30]; YData=[1 2]; ZData=[0 0 0;0 0 0]; CData=[1 2 3;4 5 6] |
| `xyc-matrices` | `struct` | `[1, 1]` | class='matlab.graphics.primitive.Surface'; XData=[10 20 30;10 20 30]; YData=[1 1 1;2 2 2]; ZData=[0 0 0;0 0 0]; CData=[1 2 3;4 5 6] |
| `no-ax-arg` | `struct` | `[1, 1]` | class='matlab.graphics.primitive.Surface'; parent_is_gca=true |
| `view` | `double` | `[1, 2]` | [0 90] |
| `edge-face` | `struct` | `[1, 1]` | FaceColor='flat'; EdgeColor=[0.12941176470588237 0.12941176470588237 0.12941176470588237] |
| `err-size` | — | — | error `MATLAB:pcolor:InputSizeMismatch` |
| `err-two-args` | — | — | error `MATLAB:badsubscript` |

## Unresolved questions

- `pcolor.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `pcolor.q-two-arg-id` — The two-argument error has a generic indexing identifier (MATLAB:badsubscript); whether an implementation must reproduce it is a human-review question (constitution 8.2).
