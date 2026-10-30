# `findstr` — behaviour specification

- **Spec version:** 0.3.0
- **Status:** approved 2026-10-29 (Tier B widening W2, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/findstr_observations.json`](observations/findstr_observations.json)

## Summary

Return the starting indices where the shorter of two strings occurs within the longer.

## Confirmed invocation forms

- `k = findstr(s1, s2)` — Start indices of the shorter argument within the longer. *(e.g. case `occurrences`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output class is double for every non-erroring observed call. *(cases: `occurrences`, `no-match`, `numeric`)*
2. **Confirmed** — Returns a row vector of the 1-based start indices of every occurrence: 'bc' within 'abcabc' gave [2 5]; 'a' within 'banana' gave [2 4 6]. *(cases: `occurrences`, `single-char`)*
3. **Confirmed** — The shorter argument is searched for within the longer regardless of argument order: findstr('bc','abcabc') also gave [2 5]. *(cases: `swapped`)*
4. **Confirmed** — Overlapping occurrences are all reported: 'aa' within 'aaaa' gave [1 2 3]. *(cases: `overlapping`)*
5. **Confirmed** — With equal-length arguments: identical 'ab' and 'ab' gave 1; 'ab' and 'ba' gave empty. *(cases: `equal-length-match`, `equal-length-nomatch`)*
6. **Confirmed** — When there is no occurrence, or one or both arguments are empty ('abc' with '', '' with 'abc', '' with ''), the result is a 0x0 empty double. *(cases: `no-match`, `equal-length-nomatch`, `empty-pattern`, `empty-first`, `both-empty`, `case`)*
7. **Confirmed** — An ordinary call issued no warning. *(cases: `warn-call`)*
8. **Confirmed** — Matching is case-sensitive: 'b' within 'ABC' gave empty. *(cases: `case`)*
9. **Confirmed** — String scalar arguments give the same result as the equivalent char arguments. *(cases: `string-scalar`)*
10. **Confirmed** — Numeric row vectors are accepted and matched element-wise: [1 2] within [1 2 3 1 2] gave [1 4]. *(cases: `numeric`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| An argument is a multi-row char array. | `MATLAB:strfind:MultiRowInput` | `column-char` |
| An argument is a cell array. | `MATLAB:findstr:InputsMustBeCharArrays` | `err-cell` |
| Only one argument is given. | `MATLAB:minrhs` | `err-one-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `occurrences` | `double` | `[1, 2]` | [2 5] |
| `swapped` | `double` | `[1, 2]` | [2 5] |
| `overlapping` | `double` | `[1, 3]` | [1 2 3] |
| `no-match` | `double` | `[0, 0]` | [] |
| `equal-length-match` | `double` | `[1, 1]` | 1 |
| `equal-length-nomatch` | `double` | `[0, 0]` | [] |
| `empty-pattern` | `double` | `[0, 0]` | [] |
| `both-empty` | `double` | `[0, 0]` | [] |
| `empty-first` | `double` | `[0, 0]` | [] |
| `warn-call` | `struct` | `[1, 1]` | value=[2 5]; warning_id='' |
| `case` | `double` | `[0, 0]` | [] |
| `single-char` | `double` | `[1, 3]` | [2 4 6] |
| `string-scalar` | `double` | `[1, 2]` | [2 5] |
| `numeric` | `double` | `[1, 2]` | [1 4] |
| `column-char` | — | — | error `MATLAB:strfind:MultiRowInput` |
| `err-cell` | — | — | error `MATLAB:findstr:InputsMustBeCharArrays` |
| `err-one-arg` | — | — | error `MATLAB:minrhs` |

## Unresolved questions

- `findstr.q-warning-coverage` — Warnings were captured only in the warn-* cases; other cases (including error and edge cases) were not checked for warnings, so an empty warnings list is not a complete statement.
- `findstr.q-deprecated` — Whether findstr is formally deprecated relative to strfind is a documentation question, not observable by black-box calls; an ordinary call issued no warning (case warn-call).

Resolved in 0.3.0: `findstr.q-order`.
