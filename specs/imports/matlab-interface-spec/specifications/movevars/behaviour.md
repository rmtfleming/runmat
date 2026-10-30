# `movevars` — behaviour specification

- **Spec version:** 0.3.0
- **Status:** approved 2026-10-29 (Tier B widening W1, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/movevars_observations.json`](observations/movevars_observations.json)

## Summary

Return a table with the specified variables moved before or after another variable, or to the end.

## Confirmed invocation forms

- `T2 = movevars(T, vars, 'Before', location)` — Move variable(s) to immediately before `location`. *(e.g. case `before`)*
- `T2 = movevars(T, vars, 'After', location)` — Move variable(s) to immediately after `location`. *(e.g. case `after-name`)*
- `T2 = movevars(T, vars)` — Move variable(s) to the end of the table. *(e.g. case `no-location`)*

## Name-value arguments

- `Before` — Location given as a variable name or numeric variable index.
- `After` — Location given as a variable name or numeric variable index.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output class is table for every non-erroring table-returning call observed. *(cases: `before`, `after-name`, `no-location`, `contents`)*
2. **Confirmed** — Moving 'c' before 'a' produced the variable order c,a,b; the variable and row counts are preserved. *(cases: `before`)*
3. **Confirmed** — `'After', location` places the moved variable immediately after `location` (a after c on a,b,c,d gave b,c,a,d; after the last variable gave b,c,d,a). *(cases: `after-name`, `after-last`)*
4. **Confirmed** — `location` may be a numeric variable index, interpreted against the input table (d Before 2 gave a,d,b,c; a After 3 gave b,c,a,d). *(cases: `before-index`, `after-index`)*
5. **Confirmed** — `vars` may be a numeric index or a logical mask (index 4 Before 1 gave d,a,b,c). *(cases: `vars-index`, `vars-logical`)*
6. **Confirmed** — Several moved variables are placed contiguously. Given as a list of names they keep the listed order ({'d','b'} Before 'a' gave d,b,a,c); given as a logical mask they keep table order (mask selecting b,d Before 1 gave b,d,a,c). *(cases: `multi-order`, `multi-after`, `vars-logical`)*
7. **Confirmed** — Moving a variable before or after itself leaves the order unchanged. *(cases: `self`, `self-after`)*
8. **Confirmed** — With no 'Before'/'After' option the variable(s) move to the end of the table, in the listed order ({'b','a'} on a,b,c,d gave c,d,b,a). *(cases: `no-location`, `no-location-multi`)*
9. **Confirmed** — Each variable's data moves with its name (column contents unchanged). *(cases: `contents`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| Both 'Before' and 'After' are supplied. | `MATLAB:table:addmovevars:BeforeAndAfter` | `err-before-and-after` |
| The variable to move is not in the table. | `MATLAB:table:UnrecognizedVarName` | `err-unknown-var` |
| The location names a variable that is not in the table. | `MATLAB:table:UnrecognizedVarName` | `err-unknown-location` |
| The location index exceeds the number of variables. | `MATLAB:table:VarIndexOutOfRange` | `err-index-range` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `before` | `table` | `[1, 3]` | c:double=3; a:double=1; b:double=2 |
| `after-name` | `table` | `[1, 4]` | b:double=2; c:double=3; a:double=1; d:double=4 |
| `after-last` | `table` | `[1, 4]` | b:double=2; c:double=3; d:double=4; a:double=1 |
| `before-index` | `table` | `[1, 4]` | a:double=1; d:double=4; b:double=2; c:double=3 |
| `after-index` | `table` | `[1, 4]` | b:double=2; c:double=3; a:double=1; d:double=4 |
| `vars-index` | `table` | `[1, 4]` | d:double=4; a:double=1; b:double=2; c:double=3 |
| `vars-logical` | `table` | `[1, 4]` | b:double=2; d:double=4; a:double=1; c:double=3 |
| `multi-order` | `table` | `[1, 4]` | d:double=4; b:double=2; a:double=1; c:double=3 |
| `multi-after` | `table` | `[1, 4]` | c:double=3; a:double=1; b:double=2; d:double=4 |
| `self` | `table` | `[1, 4]` | a:double=1; b:double=2; c:double=3; d:double=4 |
| `no-location` | `table` | `[1, 4]` | b:double=2; c:double=3; d:double=4; a:double=1 |
| `no-location-multi` | `table` | `[1, 4]` | c:double=3; d:double=4; b:double=2; a:double=1 |
| `self-after` | `table` | `[1, 4]` | a:double=1; b:double=2; c:double=3; d:double=4 |
| `contents` | `table` | `[2, 3]` | b:double=[2;5]; c:double=[3;6]; a:double=[1;4] |
| `err-before-and-after` | — | — | error `MATLAB:table:addmovevars:BeforeAndAfter` |
| `err-unknown-var` | — | — | error `MATLAB:table:UnrecognizedVarName` |
| `err-unknown-location` | — | — | error `MATLAB:table:UnrecognizedVarName` |
| `err-index-range` | — | — | error `MATLAB:table:VarIndexOutOfRange` |

## Unresolved questions

- `movevars.q-location-in-vars` — Moving several variables relative to a location that is itself one of the moved variables (only the single-variable 'self' case was exercised).

Resolved in 0.3.0: `movevars.q-after`, `movevars.q-index`.
