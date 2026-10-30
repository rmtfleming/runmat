# `removevars` — behaviour specification

- **Spec version:** 0.3.0
- **Status:** approved 2026-10-29 (Tier B widening W1, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/removevars_observations.json`](observations/removevars_observations.json)

## Summary

Return a table with the specified variables removed.

## Confirmed invocation forms

- `T2 = removevars(T, vars)` — Remove the variables selected by `vars` from table T. *(e.g. case `by-name`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output class is table for every non-erroring observed call. *(cases: `by-name`, `index`, `logical`, `cellstr`, `all`)*
2. **Confirmed** — Returns a table with the named variable removed; the remaining variables keep their original order and contents and the row count is preserved. *(cases: `by-name`, `name-full`, `string-scalar`)*
3. **Confirmed** — Numeric indices select variables by position; the order of the indices does not matter ([1 3] and [3 1] both left b,d). *(cases: `index`, `index-vector`, `index-unordered`)*
4. **Confirmed** — A logical mask selects variables to remove ([true false true false] on a,b,c,d left b,d). *(cases: `logical`)*
5. **Confirmed** — Several names may be removed at once as a cellstr or a string array; a name listed twice is removed once without error. *(cases: `cellstr`, `string-array`, `dup-names`)*
6. **Confirmed** — An empty selection ({} or []) returns the table unchanged. *(cases: `empty-cell`, `empty-index`)*
7. **Confirmed** — Removing every variable returns a table with zero variables and the original row count (size [2 0]). *(cases: `all`)*

## Edge cases (observed)

- *(non-normative)* Observed once: a logical mask shorter than the number of variables did not error; [true false] on a four-variable table removed only the first variable. *(cases: `logical-short`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A name does not match any variable. | `MATLAB:table:UnrecognizedVarName` | `err-unknown` |
| A numeric index exceeds the number of variables. | `MATLAB:table:VarIndexOutOfRange` | `err-index-range` |
| A numeric index is zero, negative, or not an integer. | `MATLAB:badsubscript` | `err-index-zero`, `err-index-negative`, `err-index-fraction` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `by-name` | `table` | `[1, 2]` | a:double=1; c:double=3 |
| `name-full` | `table` | `[2, 3]` | a:double=[1;5]; c:double=[3;7]; d:double=[4;8] |
| `index` | `table` | `[2, 3]` | a:double=[1;5]; c:double=[3;7]; d:double=[4;8] |
| `index-vector` | `table` | `[2, 2]` | b:double=[2;6]; d:double=[4;8] |
| `index-unordered` | `table` | `[2, 2]` | b:double=[2;6]; d:double=[4;8] |
| `logical` | `table` | `[2, 2]` | b:double=[2;6]; d:double=[4;8] |
| `cellstr` | `table` | `[2, 2]` | b:double=[2;6]; d:double=[4;8] |
| `string-array` | `table` | `[2, 2]` | b:double=[2;6]; d:double=[4;8] |
| `string-scalar` | `table` | `[2, 3]` | a:double=[1;5]; b:double=[2;6]; c:double=[3;7] |
| `dup-names` | `table` | `[2, 3]` | b:double=[2;6]; c:double=[3;7]; d:double=[4;8] |
| `empty-cell` | `table` | `[2, 4]` | a:double=[1;5]; b:double=[2;6]; c:double=[3;7]; d:double=[4;8] |
| `empty-index` | `table` | `[2, 4]` | a:double=[1;5]; b:double=[2;6]; c:double=[3;7]; d:double=[4;8] |
| `all` | `table` | `[2, 0]` | (no variables) |
| `err-unknown` | — | — | error `MATLAB:table:UnrecognizedVarName` |
| `err-index-range` | — | — | error `MATLAB:table:VarIndexOutOfRange` |
| `err-index-zero` | — | — | error `MATLAB:badsubscript` |
| `err-index-negative` | — | — | error `MATLAB:badsubscript` |
| `err-index-fraction` | — | — | error `MATLAB:badsubscript` |
| `logical-short` | `table` | `[2, 3]` | b:double=[2;6]; c:double=[3;7]; d:double=[4;8] |

## Unresolved questions

- `removevars.q-logical-longer` — A logical mask longer than the number of variables (not exercised).
- `removevars.q-short-mask` — Flagged for human review (constitution 8.2): whether the non-erroring short logical mask (case logical-short) should be required behaviour or left implementation-defined.

Resolved in 0.3.0: `removevars.q-index`, `removevars.q-multiple`.
