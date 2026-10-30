# `cast` — behaviour specification

- **Spec version:** 0.2.1
- **Status:** approved 2026-10-30 (correction of the 'like' complexity claim, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/cast_observations.json`](observations/cast_observations.json)

## Summary

Convert values to another numeric, logical or char class, rounding and saturating as for that class.

## Confirmed invocation forms

- `B = cast(A, newclass)` — Convert to class `newclass`. *(e.g. case `round`)*
- `B = cast(A, 'like', p)` — Convert to the class of p; a complex p (double and single observed) made the result complex. *(e.g. case `like`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Conversion to an integer class rounded to nearest with halves away from zero for the observed inputs: [1.5 2.5 -1.5] to int32 gave [2 3 -2]. *(cases: `round`)*
2. **Confirmed** — Out-of-range values saturate: [-1 300 255.6] to uint8 gave [0 255 255]; int16 [-200 5 200] to int8 gave [-128 5 127]; 70000 'like' uint16 gave 65535. *(cases: `saturate`, `int-to-int`, `like`)*
3. **Confirmed** — NaN, Inf and -Inf to int8 gave 0, 127, -128. *(cases: `nan-inf`)*
4. **Confirmed** — double to single rounds to single precision; char to double gives character codes ([97 98]); logical to double gives 1/0; double to logical gives nonzero -> true; double to char gives characters ('Hi'). *(cases: `to-single`, `char-to-double`, `logical-to-double`, `to-logical`, `to-char`)*
5. **Confirmed** — The shape is kept, and casting to the same class leaves the value unchanged. *(cases: `shape`, `same-class`)*
6. **Confirmed** — 'like' p converts to the class of p (70000 'like' uint16 gave uint16 65535). *(cases: `like`)*
7. **Confirmed** — 'like' a complex p gives a complex result of p's class even when the imaginary part is zero: cast(2, 'like', 1i) gave double 2+0i (isreal false); int8(3) 'like' 1i gave double 3+0i; 2 'like' single(1i) gave single 2+0i; [1 2] 'like' complex(1, 0) gave [1 2]+0i. *(cases: `like-complex`, `like-complex-parts`, `like-complex-int-src`, `like-complex-single`, `like-complex-zero-imag`)*
8. **Confirmed** — A complex A 'like' a real double p stayed complex: cast(1 + 2i, 'like', 1) gave 1+2i (isreal false). *(cases: `like-real-from-complex`)*

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
| `like-complex-parts` | `struct` | `[1, 1]` | class='double'; isreal=false; real=2; imag=0 |
| `like-complex-int-src` | `struct` | `[1, 1]` | class='double'; isreal=false; real=3; imag=0 |
| `like-complex-single` | `struct` | `[1, 1]` | class='single'; isreal=false; real=2; imag=0 |
| `like-complex-zero-imag` | `struct` | `[1, 1]` | class='double'; isreal=false; real=[1 2]; imag=[0 0] |
| `like-real-from-complex` | `struct` | `[1, 1]` | class='double'; isreal=false; real=1; imag=2 |
| `shape` | `uint8` | `[2, 2]` | [1 2;3 4] |
| `same-class` | `int8` | `[1, 1]` | 5 |
| `err-bad-type` | — | — | error `MATLAB:cast:UnsupportedClass` |
| `err-cell` | — | — | error `MATLAB:invalidConversion` |
| `err-struct-type` | — | — | error `MATLAB:cast:UnsupportedClass` |

## Unresolved questions

- `cast.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.

Resolved in 0.2.1: `cast.q-like-complexity`.
