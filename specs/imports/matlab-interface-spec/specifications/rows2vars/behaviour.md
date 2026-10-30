# `rows2vars` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/rows2vars_observations.json`](observations/rows2vars_observations.json)

## Summary

Transpose a table: each original variable becomes a row, each original row becomes a variable, and the first output variable lists the original variable names.

## Confirmed invocation forms

- `T2 = rows2vars(T)` — Transposed table. *(e.g. case `basic`)*
- `T2 = rows2vars(T, Name, Value)` — With VariableNamesSource and VariableNamingRule options. *(e.g. case `source-name`)*

## Name-value arguments

- `VariableNamesSource` — Variable (by name or numeric index) whose values become the new variable names; that variable is not transposed. char or string option name and value observed.
- `VariableNamingRule` — 'modify' (observed default behaviour) or 'preserve'.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — For a 2x2 table with variables x, y, the result is 2x3: OriginalVariableNames (cell {'x';'y'}), then one variable per original row holding that row's values ([1;3] and [2;4]). *(cases: `basic`)*
2. **Confirmed** — Without row names or VariableNamesSource the new variables are named Var1, Var2, ...; with row names they take the row names. *(cases: `basic`, `one-row`, `rownames`)*
3. **Confirmed** — VariableNamesSource by name or by index (char or string option name and value) takes the new names from that variable, which is then left out of the transposed rows. *(cases: `source-name`, `source-index`, `source-string`)*
4. **Confirmed** — With the default rule, or 'modify', names that are not valid identifiers are changed ('a b' became 'aB', '1c' became 'x1c') and the warning MATLAB:table:ModifiedVarnamesRows2Vars is issued (default rule). *(cases: `naming-default`, `naming-modify`, `warn-naming-default`)*
5. **Confirmed** — With VariableNamingRule 'preserve' the names are kept as given ('a b', '1c'). *(cases: `naming-preserve`)*
6. **Confirmed** — Duplicate VariableNamesSource values are made unique ('k', 'k' became k and k_1). *(cases: `dup-source`)*
7. **Confirmed** — When the original variables have different classes (double and cellstr), each new variable is a cell column. *(cases: `mixed`)*
8. **Confirmed** — A table with zero rows gives a table with only OriginalVariableNames, one row per original variable. *(cases: `empty-rows`)*

## Edge cases (observed)

- *(non-normative)* Observed: a numeric matrix input errored with MATLAB:UndefinedFunction (dispatch level; not required). *(cases: `err-not-table`)*

## Warnings (confirmed)

| condition | identifier | case |
|---|---|---|
| Names that are not valid identifiers were modified under the default naming rule. | `MATLAB:table:ModifiedVarnamesRows2Vars` | `warn-naming-default` |

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A variable has more than one column. | `MATLAB:table:rows2vars:CannotTransposeMulticolumnVar` | `err-multicol` |
| VariableNamesSource names an absent variable. | `MATLAB:table:UnrecognizedVarName` | `err-unknown-source` |
| VariableNamingRule has an unrecognised value. | `MATLAB:rows2vars:unrecognizedStringChoice` | `err-bad-rule` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `basic` | `table` | `[2, 3]` | OriginalVariableNames:cell=["'x'", "'y'"]; Var1:double=[1;3]; Var2:double=[2;4] |
| `rownames` | `table` | `[2, 3]` | OriginalVariableNames:cell=["'x'", "'y'"]; r1:double=[1;3]; r2:double=[2;4] |
| `source-name` | `table` | `[2, 3]` | OriginalVariableNames:cell=["'x'", "'y'"]; k1:double=[1;3]; k2:double=[2;4] |
| `source-index` | `table` | `[2, 3]` | OriginalVariableNames:cell=["'x'", "'y'"]; k1:double=[1;3]; k2:double=[2;4] |
| `source-string` | `table` | `[2, 3]` | OriginalVariableNames:cell=["'x'", "'y'"]; k1:double=[1;3]; k2:double=[2;4] |
| `naming-default` | `table` | `[1, 3]` | OriginalVariableNames:cell=["'x'"]; aB:double=1; x1c:double=2 |
| `naming-preserve` | `table` | `[1, 3]` | OriginalVariableNames:cell=["'x'"]; a b:double=1; 1c:double=2 |
| `warn-naming-default` | `struct` | `[1, 1]` | value={'variable_names': ['OriginalVariableNames', 'aB', 'x1c'], 'variables': [{'name': 'OriginalVariableNames', 'class': 'cell', 'size': [1, 1], 'value': ["'x'"]}, {'name': 'aB', 'class': 'double', 'size': [1, 1], 'value': '1'}, {'name': 'x1c', 'class': 'double', 'size': [1, 1], 'value': '2'}]}; warning_id='MATLAB:table:ModifiedVarnamesRows2Vars' |
| `naming-modify` | `table` | `[1, 3]` | OriginalVariableNames:cell=["'x'"]; aB:double=1; x1c:double=2 |
| `mixed` | `table` | `[2, 3]` | OriginalVariableNames:cell=["'n'", "'s'"]; Var1:cell=["1", "'p'"]; Var2:cell=["2", "'q'"] |
| `one-row` | `table` | `[2, 2]` | OriginalVariableNames:cell=["'x'", "'y'"]; Var1:double=[1;2] |
| `empty-rows` | `table` | `[2, 1]` | OriginalVariableNames:cell=["'x'", "'y'"] |
| `err-multicol` | — | — | error `MATLAB:table:rows2vars:CannotTransposeMulticolumnVar` |
| `dup-source` | `table` | `[1, 3]` | OriginalVariableNames:cell=["'x'"]; k:double=1; k_1:double=2 |
| `err-unknown-source` | — | — | error `MATLAB:table:UnrecognizedVarName` |
| `err-not-table` | — | — | error `MATLAB:UndefinedFunction` |
| `err-bad-rule` | — | — | error `MATLAB:rows2vars:unrecognizedStringChoice` |

## Unresolved questions

- `rows2vars.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `rows2vars.q-more` — Timetables, other option names and the class of mixed results for other class combinations were not exercised.
- `rows2vars.q-rownames-dup-warning` — Whether the output has row names, and whether duplicate VariableNamesSource values raise a warning, were not recorded.
