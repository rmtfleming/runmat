# `surfl` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/surfl_observations.json`](observations/surfl_observations.json)

## Summary

Draw a 3-D surface of Z and return the Surface object; its CData differs from Z (how it is computed was not established).

## Confirmed invocation forms

- `h = surfl(Z)` — Grid coordinates 1..n and 1..m (COBRA test form). *(e.g. case `z-only`)*
- `h = surfl(X, Y, Z)` — Given coordinates, vectors (COBRA form) or matrices. *(e.g. case `xyz-vectors-3x3`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is a 1x1 matlab.graphics.chart.primitive.Surface. *(cases: `z-only`, `xyz-vectors-3x3`, `xyz-vectors-4x3`, `xyz-matrices-3x3`)*
2. **Confirmed** — ZData is Z; XData and YData are full grids the size of Z (from 1..n, 1..m for surfl(Z), or expanded from vectors x, y); CData has the size of Z. *(cases: `z-only`, `xyz-vectors-3x3`, `xyz-vectors-4x3`, `xyz-matrices-3x3`)*
3. **Confirmed** — CData is not Z: for peaks(10) its values ranged from 0.3548 to 0.7787 (0.35483870967741943 to 0.77867593670498969). *(cases: `cdata-range`)*
4. **Confirmed** — After surfl the axes view was [-37.5 30]. *(cases: `view`)*
5. **Confirmed** — FaceColor is 'flat'. *(cases: `face`)*

## Edge cases (observed)

- *(non-normative)* Observed with default graphics settings: EdgeColor was [0.12941176470588237 0.12941176470588237 0.12941176470588237]; not required. *(cases: `face`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| Z has only 2 rows (with or without X and Y). | `MATLAB:surfl:InvalidInput` | `err-z-2x3`, `err-xyz-vectors-2row`, `err-xyz-matrices-2row` |
| x length does not match the columns of a 3x3 Z. | `MATLAB:xyzchk:lengthXDoesNotMatchNumColumnsZ` | `err-size` |
| A single char argument is given (surfl('abc')). | `MATLAB:narginchk:notEnoughInputs` | `err-char` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `z-only` | `struct` | `[1, 1]` | class='matlab.graphics.chart.primitive.Surface'; size=[1 1]; XData=[1 2 3;1 2 3;1 2 3]; YData=[1 1 1;2 2 2;3 3 3]; ZData=[8 1 6;3 5 7;4 9 2]; CData_size=[3 3] |
| `err-xyz-vectors-2row` | — | — | error `MATLAB:surfl:InvalidInput` |
| `err-xyz-matrices-2row` | — | — | error `MATLAB:surfl:InvalidInput` |
| `xyz-vectors-3x3` | `struct` | `[1, 1]` | class='matlab.graphics.chart.primitive.Surface'; size=[1 1]; XData=[1 2 3;1 2 3;1 2 3]; YData=[10 10 10;20 20 20;30 30 30]; ZData=[8 1 6;3 5 7;4 9 2]; CData_size=[3 3] |
| `xyz-vectors-4x3` | `struct` | `[1, 1]` | class='matlab.graphics.chart.primitive.Surface'; size=[1 1]; XData=[1 2 3;1 2 3;1 2 3;1 2 3]; YData=[10 10 10;20 20 20;30 30 30;40 40 40]; ZData=[1 5 9;2 6 10;3 7 11;4 8 12]; CData_size=[4 3] |
| `xyz-matrices-3x3` | `struct` | `[1, 1]` | class='matlab.graphics.chart.primitive.Surface'; size=[1 1]; XData=[1 2 3;1 2 3;1 2 3]; YData=[10 10 10;20 20 20;30 30 30]; ZData=[8 1 6;3 5 7;4 9 2]; CData_size=[3 3] |
| `err-z-2x3` | — | — | error `MATLAB:surfl:InvalidInput` |
| `cdata-range` | `double` | `[1, 2]` | [0.35483870967741943 0.77867593670498969] |
| `view` | `double` | `[1, 2]` | [-37.5 30] |
| `face` | `struct` | `[1, 1]` | FaceColor='flat'; EdgeColor=[0.12941176470588237 0.12941176470588237 0.12941176470588237] |
| `err-size` | — | — | error `MATLAB:xyzchk:lengthXDoesNotMatchNumColumnsZ` |
| `err-char` | — | — | error `MATLAB:narginchk:notEnoughInputs` |

## Unresolved questions

- `surfl.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `surfl.q-min-size` — The exact minimum size of Z was not established: 2 rows errored and 3 rows worked; Z with fewer than 3 columns was not tried.
- `surfl.q-cdata` — How CData is computed from the inputs was not established; only its size, and its range for one input, were recorded.
- `surfl.q-char-error` — The char-input error names argument counting (narginchk:notEnoughInputs); whether the cause is the char class or the single argument was not separated.
- `surfl.q-error-ids` — Some error identifiers name other functions (xyzchk, narginchk); whether an implementation must reproduce them is a human-review question (constitution 8.2).
