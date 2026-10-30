# `outerjoin` — behaviour specification

- **Spec version:** 0.3.0
- **Status:** approved 2026-10-29 (Tier B widening W1, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/outerjoin_observations.json`](observations/outerjoin_observations.json)

## Summary

Combine two tables by matching key variables, keeping unmatched rows from one or both tables.

## Confirmed invocation forms

- `T = outerjoin(Tleft, Tright)` — Outer join on the variables common to both tables. *(e.g. case `basic`)*
- `T = outerjoin(Tleft, Tright, 'Keys', keys)` — Join on the named or indexed key variable(s). *(e.g. case `keys-name`)*
- `T = outerjoin(Tleft, Tright, 'LeftKeys', lk, 'RightKeys', rk)` — Join on differently named key variables. *(e.g. case `leftright-keys`)*
- `T = outerjoin(___, 'MergeKeys', tf)` — Combine each key pair into a single output variable. *(e.g. case `mergekeys`)*
- `T = outerjoin(___, 'Type', type)` — Choose which unmatched rows are kept ('full', 'left', 'right'; 'inner' also accepted). *(e.g. case `type-left`)*

## Name-value arguments

- `Keys` — Key variable(s) common to both tables, by name or index.
- `LeftKeys` — Key variable(s) of Tleft; used with RightKeys.
- `RightKeys` — Key variable(s) of Tright; used with LeftKeys.
- `MergeKeys` — Logical; true merges each key pair into one variable. Default: `false`.
- `Type` — Join type. Accepted: `'full'`, `'left'`, `'right'`, `'inner'`. Default: `'full'`.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output class is table for every non-erroring observed call. *(cases: `basic`, `default-full`, `mergekeys`, `type-left`, `fill-types`)*
2. **Confirmed** — An outer join of two 2-row tables sharing one key value produced a 3-row, 4-variable table. *(cases: `basic`)*
3. **Confirmed** — Without key options, the variables with the same name in both tables are used as keys (one common variable in `default-full`; two in `multi-key`). *(cases: `default-full`, `multi-key`)*
4. **Confirmed** — `'Keys'` given as a name or as a numeric index produced the same result as key inference; `'LeftKeys'`/`'RightKeys'` join on differently named keys. *(cases: `keys-name`, `keys-index`, `leftright-keys`)*
5. **Confirmed** — Row sets by type, for left keys {1,2,4} and right keys {1,3,4}: 'full' (default) keeps matched rows plus unmatched rows of both tables (keys 1,2,3,4); 'left' keeps matched plus unmatched left rows (1,2,4); 'right' keeps matched plus unmatched right rows (1,3,4); 'inner' keeps only matched rows (1,4). *(cases: `default-full`, `type-full`, `type-left`, `type-right`, `type-inner`)*
6. **Confirmed** — Output rows are in ascending key order regardless of input row order; with two keys, rows are ordered by the first key then the second, also when both inputs are unsorted. *(cases: `unsorted`, `unsorted-merge`, `default-full`, `multi-key`, `multi-key-unsorted`)*
7. **Confirmed** — Each left row whose key matches produces its own output row: two left rows with key 1 matched to one right row with key 1 gave two output rows, in left-input order (a = 10,11 and a = 11,10 were both kept in input order). *(cases: `dup-keys`, `dup-keys-desc`)*
8. **Confirmed** — For 'full', 'left' and 'right' joins, output variables are the left table's variables in their order followed by the right table's; with 'MergeKeys' each merged key takes the position of the left key and the right key is dropped (left a,k gave a,k,b). For 'inner' see inner-key-single. *(cases: `default-full`, `type-left`, `type-right`, `mergekeys`, `multi-key`, `leftright-keys-merge`, `key-second-column`, `key-second-column-nomerge`)*
9. **Confirmed** — With 'MergeKeys' true each key pair becomes one variable holding the key value from whichever table supplied the row (no fill in the key); merging differently named keys k and id produced the name k_id. *(cases: `mergekeys`, `type-left-merge`, `unsorted-merge`, `leftright-keys-merge`)*
10. **Confirmed** — Without 'MergeKeys', a variable name present in both tables (key or non-key) is made unique by appending '_' plus the workspace variable name of each input argument (k_L/k_R; v_Ls/v_Rs; k_Lk2/k_R); when the inputs are not named workspace variables the suffixes are '_left' and '_right'. Differently named key variables keep their names. *(cases: `default-full`, `unsorted`, `same-nonkey-name`, `basic`, `leftright-keys`, `key-second-column-nomerge`)*
11. **Confirmed** — Cells with no matching row are filled with: NaN for double and single, 0 for int32, false for logical, an empty char ('') for cellstr, and <missing> for string. *(cases: `default-full`, `type-left`, `fill-types`)*
12. **Confirmed** — With 'Type','inner' (one common key, MergeKeys not given) the output held a single key variable k (k,a,b). *(cases: `type-inner`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| No variable name is common to both tables and no key options are given. | `MATLAB:table:join:CantInferKey` | `err-no-common` |
| 'Type' is not a recognised join type. | `MATLAB:table:join:InvalidType` | `err-bad-type` |
| 'Keys' is combined with 'LeftKeys'. | `MATLAB:table:join:ConflictingInputs` | `err-keys-and-leftkeys` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `basic` | `table` | `[3, 4]` | k_left:double=[1;2;NaN]; a:double=[10;20;NaN]; k_right:double=[1;NaN;3]; b:double=[100;NaN;300] |
| `default-full` | `table` | `[4, 4]` | k_L:double=[1;2;NaN;4]; a:double=[10;20;NaN;40]; k_R:double=[1;NaN;3;4]; b:double=[100;NaN;300;400] |
| `mergekeys` | `table` | `[4, 3]` | k:double=[1;2;3;4]; a:double=[10;20;NaN;40]; b:double=[100;NaN;300;400] |
| `type-full` | `table` | `[4, 4]` | k_L:double=[1;2;NaN;4]; a:double=[10;20;NaN;40]; k_R:double=[1;NaN;3;4]; b:double=[100;NaN;300;400] |
| `type-left` | `table` | `[3, 4]` | k_L:double=[1;2;4]; a:double=[10;20;40]; k_R:double=[1;NaN;4]; b:double=[100;NaN;400] |
| `type-right` | `table` | `[3, 4]` | k_L:double=[1;NaN;4]; a:double=[10;NaN;40]; k_R:double=[1;3;4]; b:double=[100;300;400] |
| `type-left-merge` | `table` | `[3, 3]` | k:double=[1;2;4]; a:double=[10;20;40]; b:double=[100;NaN;400] |
| `keys-name` | `table` | `[4, 4]` | k_L:double=[1;2;NaN;4]; a:double=[10;20;NaN;40]; k_R:double=[1;NaN;3;4]; b:double=[100;NaN;300;400] |
| `keys-index` | `table` | `[4, 4]` | k_L:double=[1;2;NaN;4]; a:double=[10;20;NaN;40]; k_R:double=[1;NaN;3;4]; b:double=[100;NaN;300;400] |
| `leftright-keys` | `table` | `[4, 4]` | k:double=[1;2;NaN;4]; a:double=[10;20;NaN;40]; id:double=[1;NaN;3;4]; b:double=[100;NaN;300;400] |
| `leftright-keys-merge` | `table` | `[4, 3]` | k_id:double=[1;2;3;4]; a:double=[10;20;NaN;40]; b:double=[100;NaN;300;400] |
| `unsorted` | `table` | `[4, 4]` | k_Lu:double=[1;2;NaN;4]; a:double=[10;20;NaN;40]; k_Ru:double=[1;NaN;3;4]; b:double=[100;NaN;300;400] |
| `unsorted-merge` | `table` | `[4, 3]` | k:double=[1;2;3;4]; a:double=[10;20;NaN;40]; b:double=[100;NaN;300;400] |
| `dup-keys` | `table` | `[5, 3]` | k:double=[1;1;2;3;4]; a:double=[10;11;20;NaN;NaN]; b:double=[100;100;NaN;300;400] |
| `same-nonkey-name` | `table` | `[3, 4]` | k_Ls:double=[1;2;NaN]; v_Ls:double=[10;20;NaN]; k_Rs:double=[1;NaN;3]; v_Rs:double=[100;NaN;300] |
| `fill-types` | `table` | `[3, 7]` | k:double=[1;2;3]; i:int32=[7;8;0]; g:logical=[true;false;false]; c:cell=["'p'", "'q'", "''"]; s:string=["s";"t";string(missing)]; f:single=[1.5;2.5;NaN]; b:double=[100;NaN;300] |
| `multi-key` | `table` | `[4, 4]` | k1:double=[1;1;2;2]; k2:double=[5;6;5;6]; a:double=[10;20;30;NaN]; b:double=[100;NaN;200;300] |
| `multi-key-unsorted` | `table` | `[4, 4]` | k1:double=[1;1;2;2]; k2:double=[5;6;5;6]; a:double=[10;20;30;NaN]; b:double=[100;NaN;200;300] |
| `dup-keys-desc` | `table` | `[5, 3]` | k:double=[1;1;2;3;4]; a:double=[11;10;20;NaN;NaN]; b:double=[100;100;NaN;300;400] |
| `key-second-column` | `table` | `[4, 3]` | a:double=[10;20;NaN;40]; k:double=[1;2;3;4]; b:double=[100;NaN;300;400] |
| `key-second-column-nomerge` | `table` | `[4, 4]` | a:double=[10;20;NaN;40]; k_Lk2:double=[1;2;NaN;4]; k_R:double=[1;NaN;3;4]; b:double=[100;NaN;300;400] |
| `err-no-common` | — | — | error `MATLAB:table:join:CantInferKey` |
| `type-inner` | `table` | `[2, 3]` | k:double=[1;4]; a:double=[10;40]; b:double=[100;400] |
| `err-bad-type` | — | — | error `MATLAB:table:join:InvalidType` |
| `err-keys-and-leftkeys` | — | — | error `MATLAB:table:join:ConflictingInputs` |

## Unresolved questions

- `outerjoin.q-index-outputs` — The second and third outputs (row indices into the left and right tables) were not observed.
- `outerjoin.q-select-variables` — The 'LeftVariables'/'RightVariables' options were not exercised.
- `outerjoin.q-many-to-many` — Duplicate keys on both sides, and missing (NaN) key values, were not exercised.
- `outerjoin.q-key-types` — Joining keys of different classes (e.g. double with int32, or string with cellstr) was not exercised.
- `outerjoin.q-name-suffix-context` — Flagged for human review (constitution 8.2): output variable names depend on the caller's workspace variable names (name-suffix). The rule is observed, but reviewers should confirm it is acceptable as required behaviour for implementations that cannot recover argument names.

Resolved in 0.3.0: `outerjoin.q-keys`, `outerjoin.q-type`.
