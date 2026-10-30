# `strread` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/strread_observations.json`](observations/strread_observations.json)

## Summary

Read formatted fields from a text value; one output per format conversion, fields assigned to outputs in cyclic order.

## Confirmed invocation forms

- `A = strread(str)` — Whitespace-separated numbers, no format. *(e.g. case `default`)*
- `A = strread(str, format)` — One conversion (%s, %d, %f). *(e.g. case `s`)*
- `[A, B] = strread(str, format, 'delimiter', d)` — Two conversions and a delimiter (two outputs observed). *(e.g. case `two-out`)*

## Name-value arguments

- `delimiter` — Field delimiter character(s) (',' and '!' observed).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — With no format, strread('1 2 3') returned the double row [1 2 3]. *(cases: `default`)*
2. **Confirmed** — %s on whitespace-separated words returns a column cell of char ({'ab';'cd';'ef'}); strread(num2str([10 2 300]), '%s') gave {'10';'2';'300'}. *(cases: `s`, `num2str-form`)*
3. **Confirmed** — %d and %f return double columns ([4;5;6], [1.5;2.25], [7;8]). *(cases: `d`, `f`, `orientation`)*
4. **Confirmed** — With two conversions, fields are assigned alternately: 'x!y!z!w' with '%s%s' and delimiter '!' gave a = {'x';'z'}, b = {'y';'w'}; 'p 1 q 2' with '%s%d' gave a = {'p';'q'}, b = [1;2]; '1 2 3' with '%d%d' gave a = [1;3], b = 2. *(cases: `two-out`, `two-out-mixed`, `two-out-uneven`)*
5. **Confirmed** — With delimiter ',', fields are split only at commas: 'a b, c d' gave {'a b';'c d'} (the inner space kept, the space after the comma not present in the field). *(cases: `delimiter-comma`, `delimiter-spaces`)*
6. **Confirmed** — Empty text with '%s' gives a 0x1 cell. *(cases: `empty`)*
7. **Confirmed** — No warning was recorded for strread('1 2'). *(cases: `warn`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The first argument is numeric. | `MATLAB:textio:dataread:SecondInputFilenameOrString` | `err-numeric` |
| Text cannot be read with the given numeric conversion. | `MATLAB:textio:dataread:TroubleReading` | `err-bad-format` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `default` | `double` | `[1, 3]` | [1 2 3] |
| `s` | `cell` | `[3, 1]` | ["'ab'", "'cd'", "'ef'"] |
| `d` | `double` | `[3, 1]` | [4;5;6] |
| `f` | `double` | `[2, 1]` | [1.5;2.25] |
| `two-out` | `struct` | `[1, 1]` | a=["'x'", "'z'"]; b=["'y'", "'w'"] |
| `two-out-mixed` | `struct` | `[1, 1]` | a=["'p'", "'q'"]; b=[1;2] |
| `two-out-uneven` | `struct` | `[1, 1]` | a=[1;3]; b=2 |
| `delimiter-comma` | `cell` | `[3, 1]` | ["'a'", "'b'", "'c'"] |
| `delimiter-spaces` | `cell` | `[2, 1]` | ["'a b'", "'c d'"] |
| `num2str-form` | `cell` | `[3, 1]` | ["'10'", "'2'", "'300'"] |
| `empty` | `cell` | `[0, 1]` | [] |
| `orientation` | `double` | `[2, 1]` | [7;8] |
| `warn` | `struct` | `[1, 1]` | value=[1 2]; warning_id='' |
| `err-numeric` | — | — | error `MATLAB:textio:dataread:SecondInputFilenameOrString` |
| `err-bad-format` | — | — | error `MATLAB:textio:dataread:TroubleReading` |

## Unresolved questions

- `strread.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `strread.q-more-options` — Options other than 'delimiter', other conversions and repeat counts were not exercised.
- `strread.q-error-ids` — The error identifiers name another function (dataread); whether an implementation must reproduce them is a human-review question (constitution 8.2).
