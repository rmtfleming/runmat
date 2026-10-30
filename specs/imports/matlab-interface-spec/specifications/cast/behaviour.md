# `cast` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/cast_observations.json`](observations/cast_observations.json)

## Summary

Convert values to another numeric, logical or char class, rounding and saturating as for that class.

## Confirmed invocation forms

- `B = cast(A, newclass)` — Convert to class `newclass`. *(e.g. case `round`)*
- `B = cast(A, 'like', p)` — Convert to the class of p. *(e.g. case `like`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Conversion to an integer class rounded to nearest with halves away from zero for the observed inputs: [1.5 2.5 -1.5] to int32 gave [2 3 -2]. *(cases: `round`)*
2. **Confirmed** — Out-of-range values saturate: [-1 300 255.6] to uint8 gave [0 255 255]; int16 [-200 5 200] to int8 gave [-128 5 127]; 70000 'like' uint16 gave 65535. *(cases: `saturate`, `int-to-int`, `like`)*
3. **Confirmed** — NaN, Inf and -Inf to int8 gave 0, 127, -128. *(cases: `nan-inf`)*
4. **Confirmed** — double to single rounds to single precision; char to double gives character codes ([97 98]); logical to double gives 1/0; double to logical gives nonzero -> true; double to char gives characters ('Hi'). *(cases: `to-single`, `char-to-double`, `logical-to-double`, `to-logical`, `to-char`)*
5. **Confirmed** — 'like' p converts to the class of p; 'like' a complex double kept a real double result (2). *(cases: `like`, `like-complex`)*
6. **Confirmed** — The shape is kept, and casting to the same class leaves the value unchanged. *(cases: `shape`, `same-class`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| newclass is not a supported class name (an unknown name, or 'struct'). | `MATLAB:cast:UnsupportedClass` | `err-bad-type`, `err-struct-type` |
| A is a cell array. | `MATLAB:invalidConversion` | `err-cell` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `round` | `int32` | `[1, 3]` | [2 3 -2] |
| `saturate` | `uint8` | `[1, 3]` | [0 255 255] |
| `nan-inf` | `int8` | `[1, 3]` | [0 127 -128] |
| `to-single` | `single` | `[1, 1]` | 0.3333333432674408 |
| `char-to-double` | `double` | `[1, 2]` | [97 98] |
| `logical-to-double` | `double` | `[1, 2]` | [1 0] |
| `int-to-int` | `int8` | `[1, 3]` | [-128 5 127] |
| `to-logical` | `logical` | `[1, 2]` | [false true] |
| `to-char` | `char` | `[1, 2]` | 'Hi' |
| `like` | `uint16` | `[1, 1]` | 65535 |
| `like-complex` | `double` | `[1, 1]` | 2 |
| `shape` | `uint8` | `[2, 2]` | [1 2;3 4] |
| `same-class` | `int8` | `[1, 1]` | 5 |
| `err-bad-type` | — | — | error `MATLAB:cast:UnsupportedClass` |
| `err-cell` | — | — | error `MATLAB:invalidConversion` |
| `err-struct-type` | — | — | error `MATLAB:cast:UnsupportedClass` |

## Unresolved questions

- `cast.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `cast.q-like-complexity` — Whether 'like' a complex value should produce a complex result is a human-review question (constitution 8.2); a real result was observed (like-complex).
