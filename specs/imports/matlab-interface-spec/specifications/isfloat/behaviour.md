# `isfloat` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isfloat_observations.json`](observations/isfloat_observations.json)

## Summary

Return true for double and single inputs and false for the other observed classes.

## Confirmed invocation forms

- `tf = isfloat(A)` — Logical scalar. *(e.g. case `double`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — True for double and single values, including complex, NaN, empty, sparse and matrix inputs (a matrix gives a scalar result). *(cases: `double`, `single`, `complex`, `nan`, `empty`, `sparse`, `matrix`)*
2. **Confirmed** — False for int8, uint64, logical, char, string, cell (even one holding a double), struct and function handle. *(cases: `int8`, `uint64`, `logical`, `char`, `string`, `cell`, `struct`, `handle`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| No input is given. | `MATLAB:minrhs` | `err-none` |
| Two inputs are given. | `MATLAB:maxrhs` | `err-two` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `double` | `logical` | `[1, 1]` | true |
| `single` | `logical` | `[1, 1]` | true |
| `complex` | `logical` | `[1, 1]` | true |
| `nan` | `logical` | `[1, 1]` | true |
| `empty` | `logical` | `[1, 1]` | true |
| `sparse` | `logical` | `[1, 1]` | true |
| `matrix` | `logical` | `[1, 1]` | true |
| `int8` | `logical` | `[1, 1]` | false |
| `uint64` | `logical` | `[1, 1]` | false |
| `logical` | `logical` | `[1, 1]` | false |
| `char` | `logical` | `[1, 1]` | false |
| `string` | `logical` | `[1, 1]` | false |
| `cell` | `logical` | `[1, 1]` | false |
| `struct` | `logical` | `[1, 1]` | false |
| `handle` | `logical` | `[1, 1]` | false |
| `err-none` | — | — | error `MATLAB:minrhs` |
| `err-two` | — | — | error `MATLAB:maxrhs` |

## Unresolved questions

- `isfloat.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
