# `strvcat` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/strvcat_observations.json`](observations/strvcat_observations.json)

## Summary

Stack text vertically into a char matrix, padding rows with trailing blanks and skipping empty inputs.

## Confirmed invocation forms

- `S = strvcat(t1, t2, ...)` — Char matrix with one row per non-empty text row. *(e.g. case `pad`)*
- `S = strvcat(c)` — Rows from a cell array of char. *(e.g. case `cell`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Rows are padded with blanks to the longest ('a','bcd','ef' gave ['a  ';'bcd';'ef '], size 3x3); trailing blanks in an input are kept. *(cases: `pad`, `pad-size`, `trailing-blanks`)*
2. **Confirmed** — Empty char inputs, including inside a cell, are skipped (three inputs with one empty gave 2 rows). *(cases: `skip-empty`, `cell-empty`)*
3. **Confirmed** — A cell array of char gives one row per element. *(cases: `cell`)*
4. **Confirmed** — A multi-row char input contributes each of its rows. *(cases: `matrix-arg`)*
5. **Confirmed** — A numeric input is taken as character codes (65 gave 'A'). *(cases: `numeric`)*
6. **Confirmed** — String scalars are accepted and give char rows. *(cases: `string`)*
7. **Confirmed** — No inputs, or only empty inputs, give a 0x0 char. *(cases: `no-args`, `all-empty`)*
8. **Confirmed** — A single char row is returned unchanged. *(cases: `one`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A char input and a cell input are mixed in one call. | `MATLAB:character:InvalidInput` | `err-mixed` |
| An input is a struct. | `MATLAB:invalidConversion` | `err-struct` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `pad` | `char` | `[3, 3]` | ['a  ';'bcd';'ef '] |
| `pad-size` | `double` | `[1, 2]` | [3 3] |
| `skip-empty` | `double` | `[1, 2]` | [2 1] |
| `cell` | `char` | `[2, 2]` | ['ab';'c '] |
| `cell-empty` | `double` | `[1, 2]` | [2 1] |
| `err-mixed` | — | — | error `MATLAB:character:InvalidInput` |
| `numeric` | `char` | `[2, 1]` | ['A';'b'] |
| `matrix-arg` | `char` | `[3, 2]` | ['ab';'cd';'e '] |
| `one` | `char` | `[1, 3]` | 'abc' |
| `all-empty` | `struct` | `[1, 1]` | class='char'; size=[0 0] |
| `no-args` | `struct` | `[1, 1]` | class='char'; size=[0 0] |
| `string` | `char` | `[2, 2]` | ['ab';'c '] |
| `trailing-blanks` | `char` | `[2, 3]` | ['a  ';'b  '] |
| `err-struct` | — | — | error `MATLAB:invalidConversion` |

## Unresolved questions

- `strvcat.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
