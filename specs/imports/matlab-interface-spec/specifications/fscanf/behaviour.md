# `fscanf` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N5, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/fscanf_observations.json`](observations/fscanf_observations.json)

## Summary

Read formatted data from an open text file: %c and %s give a char row, numeric conversions give a double column (or the requested size), and a second output counts the successful conversions.

## Confirmed invocation forms

- `A = fscanf(fid, format)` — Read the whole file with format (COBRA forms '%c' and '%s'). *(e.g. case `c-all`)*
- `A = fscanf(fid, format, sz)` — Read at most the given size (scalar n or [m Inf]). *(e.g. case `size-matrix`)*
- `[A, count] = fscanf(fid, format, ...)` — Also return the number of successful conversions. *(e.g. case `d-ints`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — '%c' returns every character, including spaces and newlines, as a 1-by-n char row; count is n (7 for 'ab c' newline 'de', 3 for 'xy' newline). *(cases: `c-all`, `c-trailing-newline`)*
2. **Confirmed** — '%s' skips whitespace and concatenates the words into one char row ('abcde'); count is the number of words (3 for the c-all text, which includes the separate word 'c'). *(cases: `s-skip`)*
3. **Confirmed** — %d, %f and %x return a double column vector ([1;2;3]; [1.5;-2;300]; '1f a' gave [31;10]); %f read across a line break. *(cases: `d-ints`, `f-reals`, `hex`)*
4. **Confirmed** — A size [2 Inf] fills a 2-row matrix column by column ([1 3 5; 2 4 6]); a scalar size n reads at most n values. *(cases: `size-matrix`, `size-scalar`)*
5. **Confirmed** — Literal text in the format is matched ('%d,' on '1,2,3' gave [1;2;3]); reading stops at the first failed conversion ('4 5 x 6' gave [4;5], count 2; 'abc' gave a 0x0 double, count 0). *(cases: `literal`, `partial`, `mismatch`)*
6. **Confirmed** — An empty file gives 0x0 char for '%c' and 0x0 double for '%d', with count 0. *(cases: `empty-c`, `empty-d`)*

## Edge cases (observed)

- Mixed numeric and %s conversions return a double column holding the character codes of the text ('%d%s' on '7 ab' gave [7; 97; 98], count 2). *(cases: `mixed-d-s`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The identifier is not an open file (-1). | `MATLAB:FileIO:InvalidFid` | `err-bad-fid` |
| No format is given. | `MATLAB:minrhs` | `err-no-format` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `c-all` | `struct` | `[1, 1]` | A=['ab c' char(10) 'de']; class='char'; size=[1 7]; count=7 |
| `s-skip` | `struct` | `[1, 1]` | A='abcde'; class='char'; size=[1 5]; count=3 |
| `c-trailing-newline` | `struct` | `[1, 1]` | A=['xy' char(10) '']; class='char'; size=[1 3]; count=3 |
| `d-ints` | `struct` | `[1, 1]` | A=[1;2;3]; class='double'; size=[3 1]; count=3 |
| `f-reals` | `struct` | `[1, 1]` | A=[1.5;-2;300]; class='double'; size=[3 1]; count=3 |
| `size-matrix` | `struct` | `[1, 1]` | A=[1 3 5;2 4 6]; class='double'; size=[2 3]; count=6 |
| `size-scalar` | `struct` | `[1, 1]` | A=[1;2]; class='double'; size=[2 1]; count=2 |
| `literal` | `struct` | `[1, 1]` | A=[1;2;3]; class='double'; size=[3 1]; count=3 |
| `mismatch` | `struct` | `[1, 1]` | A=[]; class='double'; size=[0 0]; count=0 |
| `partial` | `struct` | `[1, 1]` | A=[4;5]; class='double'; size=[2 1]; count=2 |
| `hex` | `struct` | `[1, 1]` | A=[31;10]; class='double'; size=[2 1]; count=2 |
| `mixed-d-s` | `struct` | `[1, 1]` | A=[7;97;98]; class='double'; size=[3 1]; count=2 |
| `empty-c` | `struct` | `[1, 1]` | A=''; class='char'; size=[0 0]; count=0 |
| `empty-d` | `struct` | `[1, 1]` | A=[]; class='double'; size=[0 0]; count=0 |
| `err-bad-fid` | — | — | error `MATLAB:FileIO:InvalidFid` |
| `err-no-format` | — | — | error `MATLAB:minrhs` |

## Unresolved questions

- `fscanf.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `fscanf.q-mixed` — Whether the character-code result for mixed numeric and %s conversions must be reproduced is a human-review question (constitution 8.2).
- `fscanf.q-encoding` — Only ASCII text was exercised; non-ASCII content and the encoding argument of fopen were not observed.
