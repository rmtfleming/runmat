# `validateattributes` — behaviour specification

- **Spec version:** 0.3.0
- **Status:** approved 2026-10-29 (Tier B widening W2, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/validateattributes_observations.json`](observations/validateattributes_observations.json)

## Summary

Check that an input has one of the listed classes and satisfies every listed attribute; error if not, return nothing if so.

## Confirmed invocation forms

- `validateattributes(A, classes, attributes)` — Validate A against a class list and an attribute list. *(e.g. case `pass`)*
- `validateattributes(A, classes, attributes, funcName, varName)` — As above, with funcName (used in the error identifier) and varName (accepted). *(e.g. case `fail-extra-varname`)*
- `validateattributes(A, classes, attributes, funcName, varName, argIndex)` — As above, with argIndex (accepted). *(e.g. case `pass-extra-args`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — When A satisfies the classes and attributes, the call returns without error and without a warning. *(cases: `pass`, `pass-empty-attrs`, `pass-row`, `pass-positive`, `pass-gt`, `pass-size-nan`, `pass-extra-args`)*
2. **Confirmed** — A passes the class check if its class matches any listed class: single matched {'double','single'}; char, logical, cell, struct and function_handle matched their own class names. *(cases: `pass-class-list`, `pass-class-char`, `pass-class-logical`, `pass-class-cell`, `pass-class-struct`, `pass-class-fh`)*
3. **Confirmed** — An empty attribute list imposes no attribute checks. *(cases: `pass-empty-attrs`)*
4. **Confirmed** — 'integer' accepts integer-valued doubles; 'nonnegative' accepts zero; 'nondecreasing' accepts repeated values while 'increasing' does not; '>' is strict and '>=' inclusive; NaN in a 'size' vector matches any extent. *(cases: `pass-integer-double`, `pass-nonnegative`, `pass-nondecreasing`, `fail-increasing`, `fail-gt`, `pass-ge`, `pass-size-nan`)*
5. **Confirmed** — Every listed attribute is enforced: with {'scalar','positive'}, [1 2] failed on 'scalar' and -1 failed on 'positive'. *(cases: `fail-first-of-two`, `fail-second-of-two`)*
6. **Confirmed** — With funcName given, the error identifier has funcName as its middle component (MATLAB:myfun:expectedScalar, MATLAB:myfun:invalidType). *(cases: `fail-extra-args`, `fail-extra-varname`)*

## Warnings (confirmed)

| condition | identifier | case |
|---|---|---|
| An attribute name is not recognised; the call returns without error. | `MATLAB:validateattributes:attributeNotFound` | `unknown-attribute` |

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A's class is not in the class list. | `MATLAB:invalidType` | `fail-type`, `fail-class-list` |
| The class list is empty ({}) (observed with the char input 'x'). | `MATLAB:invalidType` | `empty-class-list` |
| 'scalar' and A is not 1x1. | `MATLAB:expectedScalar` | `fail-scalar` |
| 'vector' and A is a matrix. | `MATLAB:expectedVector` | `fail-vector` |
| 'row' and A is a column. | `MATLAB:expectedRow` | `fail-row` |
| 'column' and A is a row. | `MATLAB:expectedColumn` | `fail-column` |
| '2d' and A is 3-D. | `MATLAB:expected2D` | `fail-2d` |
| 'nonempty' and A is empty. | `MATLAB:expectedNonempty` | `fail-nonempty` |
| 'square' and A is not square. | `MATLAB:expectedSquare` | `fail-square` |
| 'positive' and A contains a value that is not positive (0 and -1 observed). | `MATLAB:expectedPositive` | `fail-positive`, `fail-second-of-two` |
| 'nonnegative' and A contains a negative value. | `MATLAB:expectedNonnegative` | `fail-nonnegative` |
| 'integer' and A contains a non-integer value. | `MATLAB:expectedInteger` | `fail-integer` |
| 'finite' and A contains Inf. | `MATLAB:expectedFinite` | `fail-finite` |
| 'nonnan' and A contains NaN. | `MATLAB:expectedNonNaN` | `fail-nonnan` |
| 'real' and A is complex. | `MATLAB:expectedReal` | `fail-real` |
| 'nonzero' and A contains zero. | `MATLAB:expectedNonZero` | `fail-nonzero` |
| 'nonsparse' and A is sparse. | `MATLAB:expectedNonsparse` | `fail-nonsparse` |
| 'binary' and A contains a value other than 0 or 1. | `MATLAB:expectedBinary` | `fail-binary` |
| 'even' and A is odd. | `MATLAB:expectedEven` | `fail-even` |
| 'odd' and A is even. | `MATLAB:expectedOdd` | `fail-odd` |
| 'increasing' and A has a repeated value. | `MATLAB:expectedIncreasing` | `fail-increasing` |
| 'decreasing' and A increases. | `MATLAB:expectedDecreasing` | `fail-decreasing` |
| 'nonincreasing' and A increases. | `MATLAB:expectedNonincreasing` | `fail-nonincreasing` |
| '>', b and A contains a value equal to b. | `MATLAB:notGreater` | `fail-gt` |
| '>=', b and A contains a value below b. | `MATLAB:notGreaterEqual` | `fail-ge` |
| '<', b and A contains a value equal to b. | `MATLAB:notLess` | `fail-lt` |
| '<=', b and A contains a value above b. | `MATLAB:notLessEqual` | `fail-le` |
| 'numel', n and numel(A) differs from n. | `MATLAB:incorrectNumel` | `fail-numel` |
| 'size', sz and size(A) does not match sz. | `MATLAB:incorrectSize` | `fail-size` |
| 'ncols', n and A has a different number of columns. | `MATLAB:incorrectNumcols` | `fail-ncols` |
| 'nrows', n and A has a different number of rows. | `MATLAB:incorrectNumrows` | `fail-nrows` |
| 'ndims', n and ndims(A) differs from n. | `MATLAB:incorrectNumdims` | `fail-ndims` |
| An output argument is requested. | `MATLAB:maxlhs` | `returns-nothing` |
| With funcName 'myfun', 'scalar' fails. | `MATLAB:myfun:expectedScalar` | `fail-extra-args` |
| With funcName 'myfun', the class check fails. | `MATLAB:myfun:invalidType` | `fail-extra-varname` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `pass` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `empty-class-list` | — | — | error `MATLAB:invalidType` |
| `pass-empty-attrs` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-type` | — | — | error `MATLAB:invalidType` |
| `pass-class-list` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-class-list` | — | — | error `MATLAB:invalidType` |
| `pass-class-char` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `pass-class-logical` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `pass-class-cell` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `pass-class-struct` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `pass-class-fh` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-scalar` | — | — | error `MATLAB:expectedScalar` |
| `fail-vector` | — | — | error `MATLAB:expectedVector` |
| `pass-row` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-row` | — | — | error `MATLAB:expectedRow` |
| `fail-column` | — | — | error `MATLAB:expectedColumn` |
| `fail-2d` | — | — | error `MATLAB:expected2D` |
| `fail-nonempty` | — | — | error `MATLAB:expectedNonempty` |
| `fail-square` | — | — | error `MATLAB:expectedSquare` |
| `pass-positive` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-positive` | — | — | error `MATLAB:expectedPositive` |
| `pass-nonnegative` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-nonnegative` | — | — | error `MATLAB:expectedNonnegative` |
| `fail-integer` | — | — | error `MATLAB:expectedInteger` |
| `pass-integer-double` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-finite` | — | — | error `MATLAB:expectedFinite` |
| `fail-nonnan` | — | — | error `MATLAB:expectedNonNaN` |
| `fail-real` | — | — | error `MATLAB:expectedReal` |
| `fail-nonzero` | — | — | error `MATLAB:expectedNonZero` |
| `fail-nonsparse` | — | — | error `MATLAB:expectedNonsparse` |
| `fail-binary` | — | — | error `MATLAB:expectedBinary` |
| `fail-even` | — | — | error `MATLAB:expectedEven` |
| `fail-odd` | — | — | error `MATLAB:expectedOdd` |
| `pass-increasing` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-increasing` | — | — | error `MATLAB:expectedIncreasing` |
| `pass-nondecreasing` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-decreasing` | — | — | error `MATLAB:expectedDecreasing` |
| `fail-nonincreasing` | — | — | error `MATLAB:expectedNonincreasing` |
| `pass-gt` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-gt` | — | — | error `MATLAB:notGreater` |
| `pass-ge` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-ge` | — | — | error `MATLAB:notGreaterEqual` |
| `fail-lt` | — | — | error `MATLAB:notLess` |
| `fail-le` | — | — | error `MATLAB:notLessEqual` |
| `pass-numel` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-numel` | — | — | error `MATLAB:incorrectNumel` |
| `pass-size-nan` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-size` | — | — | error `MATLAB:incorrectSize` |
| `fail-ncols` | — | — | error `MATLAB:incorrectNumcols` |
| `fail-nrows` | — | — | error `MATLAB:incorrectNumrows` |
| `fail-ndims` | — | — | error `MATLAB:incorrectNumdims` |
| `fail-first-of-two` | — | — | error `MATLAB:expectedScalar` |
| `fail-second-of-two` | — | — | error `MATLAB:expectedPositive` |
| `unknown-attribute` | `struct` | `[1, 1]` | returned=true; warning_id='MATLAB:validateattributes:attributeNotFound' |
| `pass-extra-args` | `struct` | `[1, 1]` | returned=true; warning_id='' |
| `fail-extra-args` | — | — | error `MATLAB:myfun:expectedScalar` |
| `fail-extra-varname` | — | — | error `MATLAB:myfun:invalidType` |
| `returns-nothing` | — | — | error `MATLAB:maxlhs` |

## Unresolved questions

- `validateattributes.q-check-order` — When several attributes fail at once, which identifier is reported was not exercised.
- `validateattributes.q-more-attributes` — Attributes not exercised here (e.g. 'nonnegative' on non-numeric classes, 'diag', 'decreasing' pass cases, 'scalartext'/text attributes) remain unspecified.

Resolved in 0.3.0: `validateattributes.q-identifiers`.
