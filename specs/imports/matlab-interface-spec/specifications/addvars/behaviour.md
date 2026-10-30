# `addvars` — behaviour specification

- **Spec version:** 0.3.0
- **Status:** approved 2026-10-29 (Tier B widening W1, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/addvars_observations.json`](observations/addvars_observations.json)

## Summary

Return a table with one or more new variables inserted, appended at the end by default or placed before/after an existing variable.

## Confirmed invocation forms

- `T2 = addvars(T, var1, ..., varN)` — Append one or more variables after the last variable of T. *(e.g. case `append`)*
- `T2 = addvars(T, var1, ..., varN, 'Before', location)` — Insert the new variable(s) immediately before `location`. *(e.g. case `before-name`)*
- `T2 = addvars(T, var1, ..., varN, 'After', location)` — Insert the new variable(s) immediately after `location`. *(e.g. case `after-name`)*
- `T2 = addvars(___, 'NewVariableNames', names)` — Name the new variable(s) explicitly. *(e.g. case `newname-char`)*

## Name-value arguments

- `Before` — Location given as an existing variable name or a numeric variable index.
- `After` — Location given as an existing variable name or a numeric variable index.
- `NewVariableNames` — Names for the new variables: a char row, string scalar or cellstr for one variable; a cellstr with one name per new variable.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output class is table for every non-erroring observed call. *(cases: `append`, `append-full`, `before-name`, `newname-char`, `multi-names`)*
2. **Confirmed** — Without a location option the new variable is appended after the last existing variable; existing variables keep their order and contents and the row count is preserved. *(cases: `append`, `append-full`)*
3. **Confirmed** — Several data arguments add several variables, in argument order (T with a,b plus two columns gave a,b,Var3,Var4; with names {'y','z'} gave a,b,y,z). *(cases: `multi-default`, `multi-names`)*
4. **Confirmed** — The data argument is stored unchanged as one variable: a 2x2 numeric array became a single 2x2 variable and a 2x1 cellstr became a 2x1 cell variable. *(cases: `multicol`, `cellstr-var`)*
5. **Confirmed** — `'Before', location` inserts the new variable immediately before `location`; `location` may be a variable name or a numeric variable index (Before 'a' on a,b gave new,a,b; Before 2 gave a,new,b). *(cases: `before-name`, `before-index`, `names-and-before`)*
6. **Confirmed** — `'After', location` inserts the new variable immediately after `location`, by name or index; After the last index appends (After 'a' on a,b gave a,new,b; After 2 gave a,b,new). *(cases: `after-name`, `after-index`)*
7. **Confirmed** — `'NewVariableNames'` sets the new variable's name; a char row, a 1x1 cellstr and a string scalar gave the same result, and it combines with `'Before'`. *(cases: `newname-char`, `newname-cell`, `newname-string`, `names-and-before`, `multi-names`)*
8. **Confirmed** — When no name is given and the data argument is not a named workspace variable, the new variable is named 'Var' followed by its column position in the output table (Var2/Var3 when appended as column 2/3, Var1 when inserted first, Var2 when inserted second). *(cases: `append`, `append-full`, `before-name`, `after-name`, `before-index`, `after-index`, `multi-default`)*
9. **Confirmed** — When the data argument is passed as a named workspace variable and no name is given, the new variable takes that variable's name (a variable named x produced a table variable x). *(cases: `name-from-input`)*

## Edge cases (observed)

- *(non-normative)* Observed once: `'Before'` with an index one greater than the number of variables did not error; Before 3 on a two-variable table appended the new variable (a,b,Var3). *(cases: `before-past-end`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| Both 'Before' and 'After' are supplied. | `MATLAB:table:addmovevars:BeforeAndAfter` | `err-before-and-after` |
| A data argument has more or fewer rows than the table. | `MATLAB:table:RowDimensionMismatch` | `err-rows`, `err-rows-fewer` |
| A new variable name duplicates an existing variable name. | `MATLAB:table:DuplicateVarNames` | `err-dup-name` |
| 'NewVariableNames' supplies fewer or more names than there are new variables. | `MATLAB:table:addmovevars:IncorrectNumberOfVarNames` | `err-names-count`, `err-names-too-many` |
| The 'Before' or 'After' location names a variable that is not in the table. | `MATLAB:table:UnrecognizedVarName` | `err-bad-location`, `err-bad-location-after` |
| The 'After' location index exceeds the number of variables (After 3 on a two-variable table). | `MATLAB:table:VarIndexOutOfRange` | `err-index-range` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `append` | `table` | `[2, 2]` | a:double=[1;2]; Var2:double=[3;4] |
| `append-full` | `table` | `[2, 3]` | a:double=[1;3]; b:double=[2;4]; Var3:double=[5;6] |
| `name-from-input` | `table` | `[2, 3]` | a:double=[1;3]; b:double=[2;4]; x:double=[5;6] |
| `before-name` | `table` | `[2, 3]` | Var1:double=[5;6]; a:double=[1;3]; b:double=[2;4] |
| `after-name` | `table` | `[2, 3]` | a:double=[1;3]; Var2:double=[5;6]; b:double=[2;4] |
| `before-index` | `table` | `[2, 3]` | a:double=[1;3]; Var2:double=[5;6]; b:double=[2;4] |
| `after-index` | `table` | `[2, 3]` | a:double=[1;3]; b:double=[2;4]; Var3:double=[5;6] |
| `newname-char` | `table` | `[2, 3]` | a:double=[1;3]; b:double=[2;4]; z:double=[5;6] |
| `newname-cell` | `table` | `[2, 3]` | a:double=[1;3]; b:double=[2;4]; z:double=[5;6] |
| `newname-string` | `table` | `[2, 3]` | a:double=[1;3]; b:double=[2;4]; z:double=[5;6] |
| `multi-names` | `table` | `[2, 4]` | a:double=[1;3]; b:double=[2;4]; y:double=[5;6]; z:double=[7;8] |
| `multi-default` | `table` | `[2, 4]` | a:double=[1;3]; b:double=[2;4]; Var3:double=[5;6]; Var4:double=[7;8] |
| `names-and-before` | `table` | `[2, 3]` | a:double=[1;3]; z:double=[5;6]; b:double=[2;4] |
| `multicol` | `table` | `[2, 3]` | a:double=[1;3]; b:double=[2;4]; z:double=[5 7;6 8] |
| `cellstr-var` | `table` | `[2, 3]` | a:double=[1;3]; b:double=[2;4]; z:cell=["'p'", "'q'"] |
| `err-before-and-after` | — | — | error `MATLAB:table:addmovevars:BeforeAndAfter` |
| `err-rows` | — | — | error `MATLAB:table:RowDimensionMismatch` |
| `err-dup-name` | — | — | error `MATLAB:table:DuplicateVarNames` |
| `err-names-count` | — | — | error `MATLAB:table:addmovevars:IncorrectNumberOfVarNames` |
| `err-bad-location` | — | — | error `MATLAB:table:UnrecognizedVarName` |
| `err-index-range` | — | — | error `MATLAB:table:VarIndexOutOfRange` |
| `before-past-end` | `table` | `[2, 3]` | a:double=[1;3]; b:double=[2;4]; Var3:double=[5;6] |
| `err-bad-location-after` | — | — | error `MATLAB:table:UnrecognizedVarName` |
| `err-rows-fewer` | — | — | error `MATLAB:table:RowDimensionMismatch` |
| `err-names-too-many` | — | — | error `MATLAB:table:addmovevars:IncorrectNumberOfVarNames` |

## Unresolved questions

- `addvars.q-default-name-collision` — Default naming when the generated 'Var<n>' name already exists in the table (not exercised).
- `addvars.q-before-past-end` — Flagged for human review (constitution 8.2): whether the non-erroring 'Before' index one past the end (case before-past-end) should be required behaviour or left implementation-defined.
- `addvars.q-input-name-scope` — The workspace-name rule was observed only for a plain variable argument; behaviour for indexed or field expressions (e.g. s.x, x(:,1)) was not exercised.

Resolved in 0.3.0: `addvars.q-position`, `addvars.q-names`.
