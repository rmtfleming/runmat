# `isnat` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isnat_observations.json`](observations/isnat_observations.json)

## Summary

Return, elementwise, whether datetime values are NaT (not-a-time).

## Confirmed invocation forms

- `tf = isnat(D)` — Logical array the size of D. *(e.g. case `nat`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — NaT (including datetime('NaT')) gives true and a valid datetime false, elementwise with the shape of the input (1x3 and 2x2 observed). *(cases: `nat`, `from-text-nat`, `valid`, `array`, `matrix`)*
2. **Confirmed** — An empty 0x1 datetime gives a 0x1 logical. *(cases: `empty`)*

## Edge cases (observed)

- *(non-normative)* Observed: double, duration and char inputs errored with MATLAB:UndefinedFunction (dispatch level; not required). *(cases: `err-double`, `err-duration`, `err-char`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `nat` | `logical` | `[1, 1]` | true |
| `valid` | `logical` | `[1, 1]` | false |
| `array` | `logical` | `[1, 3]` | [false true false] |
| `matrix` | `logical` | `[2, 2]` | [false true;false false] |
| `empty` | `logical` | `[0, 1]` | zeros(0,1) |
| `from-text-nat` | `logical` | `[1, 1]` | true |
| `err-double` | — | — | error `MATLAB:UndefinedFunction` |
| `err-duration` | — | — | error `MATLAB:UndefinedFunction` |
| `err-char` | — | — | error `MATLAB:UndefinedFunction` |

## Unresolved questions

- `isnat.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
