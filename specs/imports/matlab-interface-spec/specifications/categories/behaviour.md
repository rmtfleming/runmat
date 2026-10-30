# `categories` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/categories_observations.json`](observations/categories_observations.json)

## Summary

Return the list of categories of a categorical array as a cell column of char vectors.

## Confirmed invocation forms

- `C = categories(A)` — Categories of categorical A. *(e.g. case `from-cellstr`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — For non-empty input the output is an n-by-1 cell array of char vectors. *(cases: `from-cellstr`, `numeric`, `explicit`)*
2. **Confirmed** — For the observed text inputs the categories came back sorted: b,a,c,a gave a,b,c; b,B,a gave B,a,b; a string array y,x gave x,y. *(cases: `from-cellstr`, `case`, `string-src`)*
3. **Confirmed** — An explicit category list is kept in the given order, including unused categories (c,b,a with data b,a). *(cases: `explicit`, `ordinal`)*
4. **Confirmed** — A categorical made from numbers [3 1 2 1] has categories '1', '2', '3' as char. *(cases: `numeric`)*
5. **Confirmed** — For data {'a', ''}, the empty text did not appear as a category (categories {'a'}). *(cases: `missing`)*
6. **Confirmed** — An empty categorical gives a 0x0 cell. *(cases: `empty`)*

## Edge cases (observed)

- *(non-normative)* Observed: a cellstr input errored with MATLAB:UndefinedFunction (dispatch level; not required). *(cases: `err-not-categorical`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `from-cellstr` | `cell` | `[3, 1]` | ["'a'", "'b'", "'c'"] |
| `explicit` | `cell` | `[3, 1]` | ["'c'", "'b'", "'a'"] |
| `ordinal` | `cell` | `[2, 1]` | ["'lo'", "'hi'"] |
| `numeric` | `cell` | `[3, 1]` | ["'1'", "'2'", "'3'"] |
| `missing` | `cell` | `[1, 1]` | ["'a'"] |
| `empty` | `cell` | `[0, 0]` | [] |
| `string-src` | `cell` | `[2, 1]` | ["'x'", "'y'"] |
| `case` | `cell` | `[3, 1]` | ["'B'", "'a'", "'b'"] |
| `err-not-categorical` | — | — | error `MATLAB:UndefinedFunction` |

## Unresolved questions

- `categories.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
