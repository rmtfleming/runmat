# `validatecolor` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N5, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/validatecolor_observations.json`](observations/validatecolor_observations.json)

## Summary

Validate a colour given as a name, an RGB triplet or a hexadecimal code, and return it as a double RGB row with values in [0, 1].

## Confirmed invocation forms

- `RGB = validatecolor(c)` — One colour, returned as 1x3 double (COBRA form inside any(...)). *(e.g. case `short-r`)*
- `RGB = validatecolor(c, 'multiple')` — Several colours, one row each. *(e.g. case `multiple-names`)*
- `RGB = validatecolor(c, 'one')` — Explicit single-colour option. *(e.g. case `one-option`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — validatecolor('r') returned [1 0 0], so the COBRA validator any(validatecolor('r')) is true. *(cases: `short-r`, `cobra-any`)*
2. **Confirmed** — Names: 'r' and 'red' gave [1 0 0], "blue" (string) [0 0 1], 'Green' (mixed case) [0 1 0], 'k' [0 0 0]. *(cases: `short-r`, `long-red`, `string-blue`, `mixed-case`, `short-k`)*
3. **Confirmed** — double and single triplets in [0, 1] are returned as double unchanged; a uint8 triplet is divided by 255 ([255 128 0] gave [1 0.50196 0]). *(cases: `triplet-double`, `triplet-single`, `triplet-uint8`)*
4. **Confirmed** — '#FF8000' gave [1 128/255 0]; the three-digit '#f80' gave [1 0.53333 0], where 0.53333 = 0x88/255. *(cases: `hex-6`, `hex-3`)*
5. **Confirmed** — With 'multiple', a cell of names or an m-by-3 matrix gives one row per colour ({'r','b'} gave [1 0 0; 0 0 1]); 'one' with 'r' gave [1 0 0]. *(cases: `multiple-names`, `multiple-matrix`, `one-option`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| An m-by-3 matrix with m > 1 is given without 'multiple'. | `MATLAB:graphics:validatecolor:MultipleColors` | `err-matrix-without-multiple` |
| The name is not a colour name ('notacolour', 'none'). | `MATLAB:graphics:validatecolor:InvalidColorString` | `err-unknown-name`, `err-none` |
| A double triplet component is above 1. | `MATLAB:graphics:validatecolor:OutOfRange` | `err-out-of-range` |
| A numeric input is not a triplet (two elements, or a scalar). | `MATLAB:graphics:validatecolor:InvalidTripletShape` | `err-two-components`, `err-scalar` |
| The second argument is neither 'one' nor 'multiple'. | `MATLAB:unrecognizedStringChoice` | `err-bad-option` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `short-r` | `double` | `[1, 3]` | [1 0 0] |
| `cobra-any` | `logical` | `[1, 1]` | true |
| `long-red` | `double` | `[1, 3]` | [1 0 0] |
| `string-blue` | `double` | `[1, 3]` | [0 0 1] |
| `mixed-case` | `double` | `[1, 3]` | [0 1 0] |
| `short-k` | `double` | `[1, 3]` | [0 0 0] |
| `triplet-double` | `double` | `[1, 3]` | [0.20000000000000001 0.40000000000000002 0.59999999999999998] |
| `triplet-single` | `double` | `[1, 3]` | [0.5 0.25 1] |
| `triplet-uint8` | `double` | `[1, 3]` | [1 0.50196078431372548 0] |
| `hex-6` | `double` | `[1, 3]` | [1 0.50196078431372548 0] |
| `hex-3` | `double` | `[1, 3]` | [1 0.53333333333333333 0] |
| `multiple-names` | `double` | `[2, 3]` | [1 0 0;0 0 1] |
| `multiple-matrix` | `double` | `[2, 3]` | [1 0 0;0 1 0] |
| `one-option` | `double` | `[1, 3]` | [1 0 0] |
| `err-matrix-without-multiple` | — | — | error `MATLAB:graphics:validatecolor:MultipleColors` |
| `err-unknown-name` | — | — | error `MATLAB:graphics:validatecolor:InvalidColorString` |
| `err-out-of-range` | — | — | error `MATLAB:graphics:validatecolor:OutOfRange` |
| `err-two-components` | — | — | error `MATLAB:graphics:validatecolor:InvalidTripletShape` |
| `err-none` | — | — | error `MATLAB:graphics:validatecolor:InvalidColorString` |
| `err-scalar` | — | — | error `MATLAB:graphics:validatecolor:InvalidTripletShape` |
| `err-bad-option` | — | — | error `MATLAB:unrecognizedStringChoice` |

## Unresolved questions

- `validatecolor.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `validatecolor.q-error-ids` — The bad-option identifier is generic (MATLAB:unrecognizedStringChoice); whether an implementation must reproduce it is a human-review question (constitution 8.2).
- `validatecolor.q-names` — Only the names r, red, blue, Green and k were exercised; the full set of accepted names was not enumerated.
