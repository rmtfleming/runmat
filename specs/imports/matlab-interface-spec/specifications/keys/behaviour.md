# `keys` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/keys_observations.json`](observations/keys_observations.json)

## Summary

Return the keys of a containers.Map as a sorted 1-by-n cell array.

## Confirmed invocation forms

- `k = keys(M)` — All keys of map M. *(e.g. case `char`)*
- `k = M.keys` — Method-call syntax. *(e.g. case `method-syntax`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is a 1-by-n cell array (1-by-0 for an empty map). *(cases: `char`, `numeric`, `empty`)*
2. **Confirmed** — Keys came back in sorted order for every observed input: char keys inserted as b,a,c gave a,b,c; b,B,a,A gave A,B,a,b; numeric keys inserted as 3,1,2 gave 1,2,3. *(cases: `char`, `case-order`, `numeric`)*
3. **Confirmed** — values(M) is returned in the same order as keys(M) (keys a,b,c gave values 2,1,3 for the map b->1, a->2, c->3). *(cases: `values-aligned`)*
4. **Confirmed** — M.keys gives the same result. *(cases: `method-syntax`, `char`)*

## Edge cases (observed)

- *(non-normative)* Observed: a struct input errored with MATLAB:UndefinedFunction (dispatch level; not required). *(cases: `err-not-map`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `char` | `cell` | `[1, 3]` | ["'a'", "'b'", "'c'"] |
| `numeric` | `cell` | `[1, 3]` | ['1', '2', '3'] |
| `empty` | `cell` | `[1, 0]` | [] |
| `case-order` | `cell` | `[1, 4]` | ["'A'", "'B'", "'a'", "'b'"] |
| `values-aligned` | `struct` | `[1, 1]` | keys=["'a'", "'b'", "'c'"]; values=["2", "1", "3"] |
| `method-syntax` | `cell` | `[1, 3]` | ["'a'", "'b'", "'c'"] |
| `err-not-map` | — | — | error `MATLAB:UndefinedFunction` |

## Unresolved questions

- `keys.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
