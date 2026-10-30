# `struct2cell` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/struct2cell_observations.json`](observations/struct2cell_observations.json)

## Summary

Convert a struct (array) to a cell array whose first dimension runs over the fields in field order.

## Confirmed invocation forms

- `C = struct2cell(S)` — Field values as a cell array. *(e.g. case `scalar`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — A scalar struct gives an n-by-1 cell of field values in field-definition order (fields b, a, c gave {1; 'x'; {2}}), values kept whole (matrix, nested struct). *(cases: `scalar`, `matrix-value`, `nested`)*
2. **Confirmed** — The size is [numfields, size(S)] with trailing singleton dimensions dropped: 1x2 struct with 2 fields gave 2x1x2; 2x1 with 1 field gave 1x2; 2x2 with 1 field gave 1x2x2; 0x0 with 2 fields gave 2x0x0. *(cases: `array-size`, `column-array-size`, `matrix-array-size`, `empty-array`)*
3. **Confirmed** — For a 1x2 struct array with fields a, b, C(:) was [a1, b1, a2, b2] (fields vary fastest). *(cases: `array-size`)*
4. **Confirmed** — A scalar struct with no fields gives a 0x1 cell. *(cases: `no-fields`)*

## Edge cases (observed)

- *(non-normative)* Observed: cell and double inputs errored with MATLAB:UndefinedFunction (dispatch level; not required). *(cases: `err-cell`, `err-double`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `scalar` | `cell` | `[3, 1]` | ['1', "'x'", ['2']] |
| `array-size` | `struct` | `[1, 1]` | class='cell'; size=[2 1 2]; values=["1", "3", "2", "4"] |
| `column-array-size` | `struct` | `[1, 1]` | class='cell'; size=[1 2]; values=["1", "2"] |
| `matrix-array-size` | `struct` | `[1, 1]` | class='cell'; size=[1 2 2]; values=["5", "5", "5", "5"] |
| `empty-array` | `struct` | `[1, 1]` | class='cell'; size=[2 0 0]; values=[] |
| `no-fields` | `struct` | `[1, 1]` | class='cell'; size=[0 1]; values=[] |
| `nested` | `cell` | `[1, 1]` | [{'x': '1'}] |
| `matrix-value` | `cell` | `[1, 1]` | ['[1 2;3 4]'] |
| `err-cell` | — | — | error `MATLAB:UndefinedFunction` |
| `err-double` | — | — | error `MATLAB:UndefinedFunction` |

## Unresolved questions

- `struct2cell.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
