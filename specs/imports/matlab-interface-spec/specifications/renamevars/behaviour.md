# `renamevars` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/renamevars_observations.json`](observations/renamevars_observations.json)

## Summary

Return a table with selected variables renamed; variable order, data and row names were kept in the observed cases.

## Confirmed invocation forms

- `T2 = renamevars(T, vars, newNames)` — Rename the variables selected by `vars` to `newNames`. *(e.g. case `one`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Variables can be selected by name, cellstr, string array, numeric index, index vector (paired with new names in the given order: [3 1] with {'Z','X'} renamed c to Z and a to X) and logical mask. *(cases: `one`, `many`, `string`, `index`, `index-many`, `logical`)*
2. **Confirmed** — Variable order and data are unchanged; row names r1, r2 were kept. *(cases: `one`, `many`, `rownames`)*
3. **Confirmed** — Renaming a variable to its own name is allowed and leaves the table unchanged. *(cases: `same-name`)*
4. **Confirmed** — Two names can be swapped in one call ({'a','b'} to {'b','a'}). *(cases: `swap`)*
5. **Confirmed** — A new name that is not a valid identifier ('my var') is accepted unchanged. *(cases: `non-identifier`)*

## Edge cases (observed)

- *(non-normative)* Observed: a struct input errored with MATLAB:UndefinedFunction (dispatch level; not required). *(cases: `err-not-table`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A new name equals another existing variable name. | `MATLAB:table:DuplicateVarNames` | `err-duplicate` |
| An old name is not a variable of T. | `MATLAB:table:UnrecognizedVarName` | `err-unknown` |
| The numbers of old and new names differ. | `MATLAB:table:renamevars:NumNamesMismatch` | `err-count` |
| A new name is empty. | `MATLAB:table:InvalidVarNames` | `err-empty-name` |
| A numeric index is past the last variable. | `MATLAB:table:VarIndexOutOfRange` | `err-index-range` |
| The old name is 'Row' (the row dimension name) on a table that has row names and no variable of that name. | `MATLAB:table:renamevars:RenameDim` | `err-dim-Row` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `one` | `table` | `[2, 3]` | a:double=[1;2]; B:double=[3;4]; c:double=[5;6] |
| `many` | `table` | `[2, 3]` | x:double=[1;2]; b:double=[3;4]; z:double=[5;6] |
| `index` | `table` | `[2, 3]` | a:double=[1;2]; B:double=[3;4]; c:double=[5;6] |
| `index-many` | `table` | `[2, 3]` | X:double=[1;2]; b:double=[3;4]; Z:double=[5;6] |
| `logical` | `table` | `[2, 3]` | a:double=[1;2]; B:double=[3;4]; c:double=[5;6] |
| `string` | `table` | `[2, 3]` | p:double=[1;2]; q:double=[3;4]; c:double=[5;6] |
| `same-name` | `table` | `[2, 3]` | a:double=[1;2]; b:double=[3;4]; c:double=[5;6] |
| `swap` | `table` | `[2, 3]` | b:double=[1;2]; a:double=[3;4]; c:double=[5;6] |
| `rownames` | `struct` | `[1, 1]` | table={'variable_names': ['b'], 'variables': [{'name': 'b', 'class': 'double', 'size': [2, 1], 'value': '[1;2]'}]}; RowNames=["'r1'", "'r2'"] |
| `non-identifier` | `table` | `[2, 3]` | my var:double=[1;2]; b:double=[3;4]; c:double=[5;6] |
| `err-dim-Row` | — | — | error `MATLAB:table:renamevars:RenameDim` |
| `err-duplicate` | — | — | error `MATLAB:table:DuplicateVarNames` |
| `err-unknown` | — | — | error `MATLAB:table:UnrecognizedVarName` |
| `err-count` | — | — | error `MATLAB:table:renamevars:NumNamesMismatch` |
| `err-empty-name` | — | — | error `MATLAB:table:InvalidVarNames` |
| `err-index-range` | — | — | error `MATLAB:table:VarIndexOutOfRange` |
| `err-not-table` | — | — | error `MATLAB:UndefinedFunction` |

## Unresolved questions

- `renamevars.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `renamevars.q-timetable` — Timetables and renaming of dimension names were not exercised beyond the one error case.
- `renamevars.q-row-variable` — Renaming a variable literally named 'Row' (the COBRA call renamevars(data, 'Row', 'ID')) was not observed; no case builds such a table.
