# `validatestring` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/validatestring_observations.json`](observations/validatestring_observations.json)

## Summary

Match text against a list of valid options (case-insensitive, allowing unique prefixes) and return the matching option, or error.

## Confirmed invocation forms

- `str = validatestring(txt, validStrings)` — Return the option matched by txt. *(e.g. case `exact`)*
- `str = validatestring(txt, validStrings, funcName, varName)` — With names that change the error identifier. *(e.g. case `err-funcname`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — An exact match returns that option. *(cases: `exact`)*
2. **Confirmed** — A prefix that matches exactly one option returns the full option ('ban' gave 'banana'). *(cases: `prefix`)*
3. **Confirmed** — Matching ignores case and returns the option's listed spelling ('APP' gave 'apple'). *(cases: `case`)*
4. **Confirmed** — In the two observed nested cases (the text equal to one option and a prefix of another; the text a prefix of two options where one is a prefix of the other) the shorter option was returned ('app' and 'ap' with {'app','apple'} both gave 'app'). *(cases: `exact-over-prefix`, `prefix-of-several-nested`)*
5. **Confirmed** — With a cellstr list the result is char (also for string input); with a string array list it is a string. *(cases: `exact`, `string-input`, `string-list`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The text is a prefix of two options neither of which is a prefix of the other ('b' with banana and berry). | `MATLAB:ambiguousStringChoice` | `err-ambiguous` |
| No option matches, the text is empty, or it is numeric. | `MATLAB:unrecognizedStringChoice` | `err-nomatch`, `err-empty-input`, `err-nonchar` |
| No option matches and funcName 'myfun' is given. | `MATLAB:myfun:unrecognizedStringChoice` | `err-funcname` |
| The list is a cell array of string scalars. | `MATLAB:validatestring:invalidStringList` | `err-cell-of-strings` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `exact` | `char` | `[1, 5]` | 'apple' |
| `prefix` | `char` | `[1, 6]` | 'banana' |
| `case` | `char` | `[1, 5]` | 'apple' |
| `err-cell-of-strings` | — | — | error `MATLAB:validatestring:invalidStringList` |
| `string-input` | `char` | `[1, 6]` | 'banana' |
| `string-list` | `string` | `[1, 1]` | "apple" |
| `exact-over-prefix` | `char` | `[1, 3]` | 'app' |
| `prefix-of-several-nested` | `char` | `[1, 3]` | 'app' |
| `err-ambiguous` | — | — | error `MATLAB:ambiguousStringChoice` |
| `err-nomatch` | — | — | error `MATLAB:unrecognizedStringChoice` |
| `err-funcname` | — | — | error `MATLAB:myfun:unrecognizedStringChoice` |
| `err-nonchar` | — | — | error `MATLAB:unrecognizedStringChoice` |
| `err-empty-input` | — | — | error `MATLAB:unrecognizedStringChoice` |

## Unresolved questions

- `validatestring.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
- `validatestring.q-more-args` — Forms with more than four input arguments were not exercised.
