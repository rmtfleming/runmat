# `amd` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/amd_observations.json`](observations/amd_observations.json)

## Summary

Return a fill-reducing symmetric ordering (permutation vector) of a square matrix.

## Confirmed invocation forms

- `p = amd(A)` — Approximate minimum degree permutation of square A. *(e.g. case `perm`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is a double row vector of size 1-by-n. *(cases: `perm`, `class`, `diag`)*
2. **Confirmed** — The output is a permutation of 1:n for every observed square input (5x5 banded, 4x4 arrow, 3x3 diagonal). *(cases: `perm`, `arrow`, `diag`)*
3. **Confirmed** — Full input gave the same permutation as the equivalent sparse input (4x4 arrow). *(cases: `arrow`, `dense-input`)*
4. **Confirmed** — 0x0 input gives a 1x0 empty double; 1x1 input gives 1. *(cases: `empty`, `scalar`)*

## Edge cases (observed)

- *(non-normative)* The specific orderings observed ([5 4 1 2 3] for the 5x5 banded matrix, [4 3 2 1] for the arrow, [1 2 3] for the diagonal) are recorded for reference; only the permutation property is required. *(cases: `perm`, `arrow`, `diag`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A is not square. | `MATLAB:square` | `err-nonsquare` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `perm` | `struct` | `[1, 1]` | p=[5 4 1 2 3]; is_permutation=true; size=[1 5]; class='double' |
| `arrow` | `struct` | `[1, 1]` | p=[4 3 2 1]; is_permutation=true; size=[1 4]; class='double' |
| `dense-input` | `struct` | `[1, 1]` | p=[4 3 2 1]; is_permutation=true; size=[1 4]; class='double' |
| `class` | `double` | `[1, 4]` | [4 3 1 2] |
| `diag` | `struct` | `[1, 1]` | p=[1 2 3]; is_permutation=true; size=[1 3]; class='double' |
| `empty` | `double` | `[1, 0]` | zeros(1,0) |
| `scalar` | `double` | `[1, 1]` | 1 |
| `err-nonsquare` | — | — | error `MATLAB:square` |

## Unresolved questions

- `amd.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
- `amd.q-more-args` — Forms with arguments beyond A were not exercised.
- `amd.q-ordering` — Whether conforming implementations must reproduce the exact ordering is a human-review question (constitution 8.2); the spec requires only a valid permutation.
