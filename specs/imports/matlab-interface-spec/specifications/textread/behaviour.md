# `textread` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/textread_observations.json`](observations/textread_observations.json)

## Summary

Read formatted data from a text file into separate output variables, one per format field.

## Confirmed invocation forms

- `[A, B, ...] = textread(filename, format)` — One output per non-skipped format field. *(e.g. case `numeric`)*
- `[___] = textread(filename, format, Name, Value)` — With header-line or delimiter options. *(e.g. case `headerlines`)*
- `A = textread(filename)` — Without a format. *(e.g. case `default-format`)*

## Name-value arguments

- `headerlines` — Number of initial lines to skip.
- `delimiter` — Field delimiter (',' observed).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Each format field fills one output as a column: '1 2\n3 4\n' with '%f %f' gave [1;3] and [2;4]. *(cases: `numeric`, `single-column`)*
2. **Confirmed** — %f and %d fields give double columns; %s fields give cell column arrays of char. *(cases: `numeric`, `mixed`)*
3. **Confirmed** — 'headerlines', 1 skips the first line; 'delimiter', ',' splits on commas. *(cases: `headerlines`, `delimiter`)*
4. **Confirmed** — %*s reads and discards a field, so it produces no output. *(cases: `skip-field`)*
5. **Confirmed** — With no format and one output, the numbers were returned as a matrix with the file's row and column layout ([1 2; 3 4]). *(cases: `default-format`)*
6. **Confirmed** — An empty file with '%f' gives a 0x1 double. *(cases: `empty-file`)*
7. **Confirmed** — No warning was issued in the probed cases. *(cases: `numeric`, `mixed`, `headerlines`, `delimiter`, `single-column`, `default-format`, `empty-file`, `skip-field`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The file does not exist. | `MATLAB:textread:FileNotFound` | `err-missing` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `numeric` | `struct` | `[1, 1]` | warning_id=''; out1=[1;3]; class1='double'; size1=[2 1]; out2=[2;4]; class2='double'; size2=[2 1] |
| `mixed` | `struct` | `[1, 1]` | warning_id=''; out1=["'x'", "'y'"]; class1='cell'; size1=[2 1]; out2=[1;2]; class2='double'; size2=[2 1] |
| `headerlines` | `struct` | `[1, 1]` | warning_id=''; out1=[5;6]; class1='double'; size1=[2 1] |
| `delimiter` | `struct` | `[1, 1]` | warning_id=''; out1=[1;3]; class1='double'; size1=[2 1]; out2=[2;4]; class2='double'; size2=[2 1] |
| `single-column` | `struct` | `[1, 1]` | warning_id=''; out1=[7;8;9]; class1='double'; size1=[3 1] |
| `default-format` | `struct` | `[1, 1]` | warning_id=''; out1=[1 2;3 4]; class1='double'; size1=[2 2] |
| `empty-file` | `struct` | `[1, 1]` | warning_id=''; out1=zeros(0,1); class1='double'; size1=[0 1] |
| `skip-field` | `struct` | `[1, 1]` | warning_id=''; out1=[1;2]; class1='double'; size1=[2 1] |
| `err-missing` | — | — | error `MATLAB:textread:FileNotFound` |

## Unresolved questions

- `textread.q-more-formats` — Other format specifiers and options were not exercised.
