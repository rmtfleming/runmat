# `convertvars` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/convertvars_observations.json`](observations/convertvars_observations.json)

## Summary

Return a table with selected variables converted to a named type or by a function; other variables are unchanged.

## Confirmed invocation forms

- `T2 = convertvars(T, vars, type)` — Convert the selected variables to `type` (COBRA form with a string array of names and 'string'). *(e.g. case `string`)*
- `T2 = convertvars(T, vars, func)` — Convert the selected variables by applying a function. *(e.g. case `function`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Variables can be selected by name, string array of names, numeric index and logical mask; unselected variables are unchanged. *(cases: `string`, `string-array-names`, `index`, `mask`)*
2. **Confirmed** — convertvars(T, ["Reaction ID" "Taxon"], 'string') converted both cellstr variables (names containing a space) to string columns. *(cases: `preserve-names`)*
3. **Confirmed** — cellstr to 'string' gives string; cellstr to 'categorical' gives categorical (categories a, b); double to 'string' gives "1", "2"; double to 'int8' gives int8; double to 'double' left the table unchanged. *(cases: `string`, `categorical`, `numeric-to-string`, `to-int8`, `same-type`)*
4. **Confirmed** — A function handle (@upper) is applied to the variable (cellstr {'a';'b'} became {'A';'B'}, still cell). *(cases: `function`)*

## Edge cases (observed)

- *(non-normative)* Observed: a struct input errored with MATLAB:UndefinedFunction (dispatch level; not required). *(cases: `err-not-table`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A selected variable is not in T. | `MATLAB:table:UnrecognizedVarName` | `err-unknown` |
| The type name is not recognised. | `MATLAB:table:convertvars:InvalidTextType` | `err-bad-type` |
| The conversion fails (cellstr to 'double'). | `MATLAB:table:convertvars:VariableTypesConversionFailed` | `err-cell-to-double` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `string` | `table` | `[2, 3]` | p:string=["a";"b"]; n:double=[1;2]; q:cell=["'x'", "'y'"] |
| `string-array-names` | `table` | `[2, 3]` | p:string=["a";"b"]; n:double=[1;2]; q:string=["x";"y"] |
| `preserve-names` | `table` | `[2, 2]` | Reaction ID:string=["a";"b"]; Taxon:string=["c";"d"] |
| `index` | `table` | `[2, 3]` | p:cell=["'a'", "'b'"]; n:double=[1;2]; q:string=["x";"y"] |
| `mask` | `table` | `[2, 3]` | p:string=["a";"b"]; n:double=[1;2]; q:cell=["'x'", "'y'"] |
| `categorical` | `table` | `[2, 3]` | p:categorical={'categorical_values': ['a', 'b'], 'categories': ['a', 'b']}; n:double=[1;2]; q:cell=["'x'", "'y'"] |
| `numeric-to-string` | `table` | `[2, 3]` | p:cell=["'a'", "'b'"]; n:string=["1";"2"]; q:cell=["'x'", "'y'"] |
| `to-int8` | `table` | `[2, 3]` | p:cell=["'a'", "'b'"]; n:int8=[1;2]; q:cell=["'x'", "'y'"] |
| `function` | `table` | `[2, 3]` | p:cell=["'A'", "'B'"]; n:double=[1;2]; q:cell=["'x'", "'y'"] |
| `same-type` | `table` | `[2, 3]` | p:cell=["'a'", "'b'"]; n:double=[1;2]; q:cell=["'x'", "'y'"] |
| `err-unknown` | — | — | error `MATLAB:table:UnrecognizedVarName` |
| `err-bad-type` | — | — | error `MATLAB:table:convertvars:InvalidTextType` |
| `err-cell-to-double` | — | — | error `MATLAB:table:convertvars:VariableTypesConversionFailed` |
| `err-not-table` | — | — | error `MATLAB:UndefinedFunction` |

## Unresolved questions

- `convertvars.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
