# `dec2base` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/dec2base_observations.json`](observations/dec2base_observations.json)

## Summary

Convert non-negative integers to text in a given base (2-36), using digits 0-9 then upper-case letters.

## Confirmed invocation forms

- `str = dec2base(d, base)` — Text representation of d in base `base`. *(e.g. case `base2`)*
- `str = dec2base(d, base, n)` — With at least n digits. *(e.g. case `pad`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is a char array. *(cases: `base2`, `vector`)*
2. **Confirmed** — Digits above 9 are upper-case letters: 23 in base 2 gave '10111', in base 3 '212'; 255 in base 16 gave 'FF'; 200 in base 16 gave 'C8'; 35 in base 36 gave 'Z'. *(cases: `base2`, `base3`, `base16`, `uint8`, `base36-z`)*
3. **Confirmed** — 0 gives '0'. *(cases: `zero`)*
4. **Confirmed** — n pads with leading zeros to at least n digits ('00000101'); n smaller than needed does not truncate ('11111111'). *(cases: `pad`, `pad-short`)*
5. **Confirmed** — Several values give one row per element, padded with leading zeros to a common width, for both a column vector ([1;10;100] in base 10 gave ['001';'010';'100']) and a row vector ([1 2 3] in base 2 gave ['01';'10';'11']). *(cases: `vector`, `row-vector`, `base36`)*
6. **Confirmed** — Integer-class input is accepted (uint8(200) in base 16 gave 'C8'). *(cases: `uint8`)*
7. **Confirmed** — The double 2^52 in base 16 gave '10000000000000'. *(cases: `large`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| d is negative or not an integer. | `MATLAB:dec2base:FirstArg` | `err-negative`, `err-fraction` |
| base is 1 or 37 (outside 2-36), or the two-character char array 'ab'. | `MATLAB:dec2base:SecondArg` | `err-base1`, `err-base37`, `err-symbols` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `base2` | `char` | `[1, 5]` | '10111' |
| `base3` | `char` | `[1, 3]` | '212' |
| `base16` | `char` | `[1, 2]` | 'FF' |
| `base36` | `char` | `[2, 2]` | ['0Z';'10'] |
| `zero` | `char` | `[1, 1]` | '0' |
| `base36-z` | `char` | `[1, 1]` | 'Z' |
| `pad` | `char` | `[1, 8]` | '00000101' |
| `pad-short` | `char` | `[1, 8]` | '11111111' |
| `vector` | `char` | `[3, 3]` | ['001';'010';'100'] |
| `row-vector` | `char` | `[3, 2]` | ['01';'10';'11'] |
| `err-symbols` | — | — | error `MATLAB:dec2base:SecondArg` |
| `uint8` | `char` | `[1, 2]` | 'C8' |
| `large` | `char` | `[1, 14]` | '10000000000000' |
| `err-negative` | — | — | error `MATLAB:dec2base:FirstArg` |
| `err-fraction` | — | — | error `MATLAB:dec2base:FirstArg` |
| `err-base1` | — | — | error `MATLAB:dec2base:SecondArg` |
| `err-base37` | — | — | error `MATLAB:dec2base:SecondArg` |

## Unresolved questions

- `dec2base.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
- `dec2base.q-large-values` — Values beyond 2^52 were not exercised.
- `dec2base.q-char-base` — Flagged for human review (constitution 8.2): char values for the base argument were observed only as the rejected 'ab'; other char forms were not exercised.
