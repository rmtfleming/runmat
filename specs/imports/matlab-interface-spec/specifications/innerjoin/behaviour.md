# `innerjoin` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/innerjoin_observations.json`](observations/innerjoin_observations.json)

## Summary

Combine two tables, keeping only rows whose key values appear in both.

## Confirmed invocation forms

- `T = innerjoin(Tleft, Tright)` — Inner join on the common variables. *(e.g. case `basic`)*
- `T = innerjoin(Tleft, Tright, Name, Value)` — With key or variable-selection options. *(e.g. case `keys`)*

## Name-value arguments

- `Keys` — Key variable(s) common to both tables.
- `LeftKeys` — Key variable(s) of Tleft; with RightKeys.
- `RightKeys` — Key variable(s) of Tright; with LeftKeys.
- `LeftVariables` — Variables of Tleft to include in the output.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is a table for every non-erroring observed call. *(cases: `basic`, `no-match`)*
2. **Confirmed** — Only rows whose key appears in both tables are kept (left keys {1,2,4}, right keys {4,1,3} gave keys [1;4]). *(cases: `basic`)*
3. **Confirmed** — Rows came out in ascending key order in every observed case, including an unsorted left table ([1;4]) and two unsorted tables ([1;2;3]). *(cases: `basic`, `unsorted-left`, `unsorted-both`)*
4. **Confirmed** — The key appears once in the output, followed by the left non-key variables, then the right non-key variables (k, a, b). *(cases: `basic`, `keys`)*
5. **Confirmed** — Keys by name gives the same result as inference; LeftKeys/RightKeys join differently named keys, and the output keeps the left key name (k). *(cases: `keys`, `leftright-keys`)*
6. **Confirmed** — Each matching left row produced its own row; for the observed pair of equal keys the left-input order was kept (a = 11 then 10). *(cases: `dup-left`)*
7. **Confirmed** — A non-key name present in both tables is made unique by appending '_' plus the workspace variable name of each input (v_Lw, v_Rw); when the inputs are not named variables the suffixes are '_left' and '_right'. *(cases: `same-nonkey`, `same-nonkey-named`)*
8. **Confirmed** — With no common key values the result is a 0-row table with all variables. *(cases: `no-match`)*
9. **Confirmed** — LeftVariables 'k' kept only k from the left table, followed by the right non-key variable (k, b). *(cases: `left-variables`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| No variable is common to both tables and no keys are given. | `MATLAB:table:join:CantInferKey` | `err-no-common` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `basic` | `table` | `[2, 3]` | k:double=[1;4]; a:double=[10;40]; b:double=[100;400] |
| `keys` | `table` | `[2, 3]` | k:double=[1;4]; a:double=[10;40]; b:double=[100;400] |
| `leftright-keys` | `table` | `[2, 3]` | k:double=[1;4]; a:double=[10;40]; b:double=[100;400] |
| `unsorted-left` | `table` | `[2, 3]` | k:double=[1;4]; a:double=[10;40]; b:double=[100;400] |
| `unsorted-both` | `table` | `[3, 3]` | k:double=[1;2;3]; a:double=[10;20;30]; b:double=[100;200;300] |
| `dup-left` | `table` | `[3, 3]` | k:double=[1;1;4]; a:double=[11;10;40]; b:double=[100;100;400] |
| `same-nonkey` | `table` | `[2, 3]` | k:double=[1;2]; v_left:double=[10;20]; v_right:double=[100;200] |
| `same-nonkey-named` | `table` | `[2, 3]` | k:double=[1;2]; v_Lw:double=[10;20]; v_Rw:double=[100;200] |
| `no-match` | `table` | `[0, 3]` | k:double=zeros(0,1); a:double=zeros(0,1); b:double=zeros(0,1) |
| `left-variables` | `table` | `[2, 2]` | k:double=[1;4]; b:double=[100;400] |
| `err-no-common` | — | — | error `MATLAB:table:join:CantInferKey` |

## Unresolved questions

- `innerjoin.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
- `innerjoin.q-name-suffix-context` — Flagged for human review (constitution 8.2), as for outerjoin: output names depend on the caller's workspace variable names (name-suffix).
- `innerjoin.q-index-outputs` — The second and third outputs (row indices) were not observed.
- `innerjoin.q-many-to-many` — Duplicate keys on both sides were not exercised.
