# `hsv` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/hsv_observations.json`](observations/hsv_observations.json)

## Summary

Return the hue-saturation-value colormap as an n-by-3 matrix of RGB values.

## Confirmed invocation forms

- `c = hsv(n)` — n-by-3 colormap. *(e.g. case `six`)*
- `c = hsv` — Colormap of the default length. *(e.g. case `default-size`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is an n-by-3 double matrix. *(cases: `six`, `four`, `class`)*
2. **Confirmed** — hsv(6) gave [1 0 0; 1 1 0; 0 1 0; 0 1 1; 0 0 1; 1 0 1]; hsv(4) gave [1 0 0; 0.5 1 0; 0 1 1; 0.5 0 1]; hsv(1) gave [1 0 0]. *(cases: `six`, `four`, `one`)*
3. **Confirmed** — hsv(64) has values in [0, 1], first row [1 0 0] and last row [1 0 0.09375]. *(cases: `range`)*
4. **Confirmed** — hsv(0) is a 0-by-3 empty matrix. *(cases: `zero`)*
5. **Confirmed** — With no argument and no figure open, the result was 256-by-3. *(cases: `default-size`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| n is negative. | `MATLAB:validators:mustBeNonnegative` | `err-negative` |
| n is not an integer. | `MATLAB:validators:mustBeInteger` | `err-fraction` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `six` | `double` | `[6, 3]` | [1 0 0;1 1 0;0 1 0;0 1 1;0 0 1;1 0 1] |
| `four` | `double` | `[4, 3]` | [1 0 0;0.5 1 0;0 1 1;0.5 0 1] |
| `one` | `double` | `[1, 3]` | [1 0 0] |
| `zero` | `double` | `[0, 3]` | zeros(0,3) |
| `default-size` | `double` | `[1, 2]` | [256 3] |
| `range` | `struct` | `[1, 1]` | size=[64 3]; min=0; max=1; first=[1 0 0]; last=[1 0 0.09375] |
| `class` | `char` | `[1, 6]` | 'double' |
| `err-negative` | — | — | error `MATLAB:validators:mustBeNonnegative` |
| `err-fraction` | — | — | error `MATLAB:validators:mustBeInteger` |

## Unresolved questions

- `hsv.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `hsv.q-default-with-figure` — The default length when a figure with a colormap is open was not exercised.
