# `asManyOfPattern` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/asManyOfPattern_observations.json`](observations/asManyOfPattern_observations.json)

## Summary

Build a text pattern that matches as many consecutive repetitions of a pattern as possible, with optional minimum and maximum counts.

## Confirmed invocation forms

- `pat = asManyOfPattern(p)` — Zero or more repetitions. *(e.g. case `default-min`)*
- `pat = asManyOfPattern(p, min)` — At least `min` repetitions (COBRA form with min 1). *(e.g. case `cobra-extract`)*
- `pat = asManyOfPattern(p, min, max)` — Between `min` and `max` repetitions. *(e.g. case `min-max`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The result is of class pattern. *(cases: `class`)*
2. **Confirmed** — extract("ab_1 cd-e", asManyOfPattern(alphanumericsPattern | "_", 1)) gave ["ab_1"; "cd"; "e"]; on "--" it gave a 0x1 string; matches(["ab_1" "ab 1" ""], ...) gave [true false false]; contains(["--" "-a-"], ...) gave [false true]. *(cases: `cobra-extract`, `min1-no-empty`, `matches`, `contains`)*
3. **Confirmed** — Repetitions are taken as long as possible: extract("ab cd", asManyOfPattern(lettersPattern(1))) gave ["ab"; "cd"]. *(cases: `default-min`)*
4. **Confirmed** — With min 2 and max 3 on digits, "1 22 333 4444" gave ["22"; "333"; "444"] (a single digit not matched; 4444 matched as 444). *(cases: `min-max`)*
5. **Confirmed** — Text is accepted as the pattern: with min 2, "a aa aaa" gave ["aa"; "aaa"] for both string "a" and char 'a'. *(cases: `string-input-min`, `char-input`)*

## Edge cases (observed)

- *(non-normative)* Observed, not required until q-empty-match is resolved: with the default minimum, extract("aaab", asManyOfPattern("a")) gave ["aaa"; ""] and extract("ab cd", asManyOfPattern("b")) gave [""; "b"; ""; ""; ""]. *(cases: `string-input`, `default-min-string-letters`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| min is greater than max. | `MATLAB:pattern:InvalidLimitRange` | `err-min-gt-max` |
| min is negative. | `MATLAB:pattern:MustBeNonnegativeInteger` | `err-negative` |
| p is numeric. | `MATLAB:pattern:MustBeTextOrPattern` | `err-numeric` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `class` | `char` | `[1, 7]` | 'pattern' |
| `cobra-extract` | `string` | `[3, 1]` | ["ab_1";"cd";"e"] |
| `min1-no-empty` | `string` | `[0, 1]` | strings(0,1) |
| `default-min` | `string` | `[2, 1]` | ["ab";"cd"] |
| `min-max` | `string` | `[3, 1]` | ["22";"333";"444"] |
| `matches` | `logical` | `[1, 3]` | [true false false] |
| `contains` | `logical` | `[1, 2]` | [false true] |
| `string-input` | `string` | `[2, 1]` | ["aaa";""] |
| `string-input-min` | `string` | `[2, 1]` | ["aa";"aaa"] |
| `char-input` | `string` | `[2, 1]` | ["aa";"aaa"] |
| `default-min-string-letters` | `string` | `[5, 1]` | ["";"b";"";"";""] |
| `err-min-gt-max` | — | — | error `MATLAB:pattern:InvalidLimitRange` |
| `err-negative` | — | — | error `MATLAB:pattern:MustBeNonnegativeInteger` |
| `err-numeric` | — | — | error `MATLAB:pattern:MustBeTextOrPattern` |

## Unresolved questions

- `asManyOfPattern.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `asManyOfPattern.q-empty-match` — Empty matches with the default minimum are not consistent across the observed cases and need review (constitution 8.2): extract("aaab", asManyOfPattern("a")) gave ["aaa"; ""] and extract("ab cd", asManyOfPattern("b")) gave [""; "b"; ""; ""; ""], but extract("ab cd", asManyOfPattern(lettersPattern(1))) gave ["ab"; "cd"] with none. The rule deciding where empty matches appear was not established.
