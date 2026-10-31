# `varfun` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/varfun_observations.json`](observations/varfun_observations.json)

## Summary

Apply a function to each variable of a table and return a table of the results with variables named <function>_<variable>.

## Confirmed invocation forms

- `R = varfun(func, T)` — Apply func to each variable of T (COBRA form, e.g. varfun(@mean, T)). *(e.g. case `mean`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Output variables are named <function name>_<variable name> (mean_x, std_y, cumsum_x, max_v); an anonymous function gives Fun_<variable>. *(cases: `mean`, `std`, `elementwise`, `one-var`, `anonymous`)*
2. **Confirmed** — A scalar-valued function gives a one-row table (mean of [1;2;3] and [4;6;8] gave 2 and 6; std gave 1 and 2); a function returning a column gives that many rows (cumsum, v*2 gave 3 rows). *(cases: `mean`, `std`, `elementwise`, `anonymous`)*
3. **Confirmed** — table2array(varfun(@mean, T))' gave the column [2;6]; varfun on T([1 3], 2:end) gave mean_y = 6. *(cases: `table2array`, `subset`)*
4. **Confirmed** — For a table with row names and a scalar-valued function the result had one row with variable mean_x (row names of the result were not recorded). *(cases: `rownames`)*
5. **Confirmed** — A zero-row table with @sum gave a one-row result sum_x = 0. *(cases: `empty-rows`)*

## Edge cases (observed)

- *(non-normative)* Observed: a numeric matrix input errored with MATLAB:UndefinedFunction (dispatch level; not required). *(cases: `err-not-table`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| func fails on a variable (here @mean on a cellstr variable). | `MATLAB:table:varfun:FunFailed` | `err-cellstr` |
| The first argument is numeric (varfun(5, T) observed). | `MATLAB:table:varfun:InvalidFunction` | `err-not-function` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `mean` | `table` | `[1, 2]` | mean_x:double=2; mean_y:double=6 |
| `std` | `table` | `[1, 2]` | std_x:double=1; std_y:double=2 |
| `table2array` | `double` | `[2, 1]` | [2;6] |
| `subset` | `table` | `[1, 1]` | mean_y:double=6 |
| `anonymous` | `table` | `[3, 2]` | Fun_x:double=[2;4;6]; Fun_y:double=[8;12;16] |
| `elementwise` | `table` | `[3, 2]` | cumsum_x:double=[1;3;6]; cumsum_y:double=[4;10;18] |
| `rownames` | `table` | `[1, 1]` | mean_x:double=2 |
| `empty-rows` | `table` | `[1, 1]` | sum_x:double=0 |
| `one-var` | `table` | `[1, 1]` | max_v:double=5 |
| `err-cellstr` | — | — | error `MATLAB:table:varfun:FunFailed` |
| `err-not-table` | — | — | error `MATLAB:UndefinedFunction` |
| `err-not-function` | — | — | error `MATLAB:table:varfun:InvalidFunction` |

## Unresolved questions

- `varfun.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `varfun.q-more-args` — Calls with more than two arguments were not exercised; COBRA uses only the two-argument form.
