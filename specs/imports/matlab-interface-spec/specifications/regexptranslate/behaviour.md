# `regexptranslate` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/regexptranslate_observations.json`](observations/regexptranslate_observations.json)

## Summary

Translate text into a regular-expression pattern: escape special characters ('escape') or convert wildcards ('wildcard').

## Confirmed invocation forms

- `p = regexptranslate(op, s)` — Translate text s using operation op. *(e.g. case `escape-dot`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — 'escape' prefixed each of the characters . $ ^ * + ? ( ) [ ] { } | \ and - with a backslash ('a.b' gave 'a\.b'); letters and the characters < > # / space , : = ! @ were left unchanged. *(cases: `escape-dot`, `escape-specials`, `escape-plain`, `escape-others`)*
2. **Confirmed** — 'escape' left a tab and a newline character unchanged. *(cases: `escape-whitespace`)*
3. **Confirmed** — 'wildcard' turns * into .*, ? into ., and escapes a literal dot: '*.m' gave '.*\.m'; 'a?c' gave 'a.c'; 'a.b' gave 'a\.b'. *(cases: `wildcard-star`, `wildcard-question`, `wildcard-dot`)*
4. **Confirmed** — The translated wildcard pattern, anchored with ^ and $, matched 'file.m' using regexp. *(cases: `wildcard-use`)*
5. **Confirmed** — A string scalar input gives a string; a cell array gives a cell array translated element-wise; empty text gives empty char. *(cases: `string-input`, `cell-input`, `empty`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| op is not a recognised operation. | `MATLAB:REGEXP:invalidOptions` | `err-op` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `escape-dot` | `char` | `[1, 4]` | 'a\.b' |
| `escape-specials` | `char` | `[1, 26]` | '\$\^\*\+\?\(\)\[\]\{\}\/\\' |
| `escape-plain` | `char` | `[1, 3]` | 'abc' |
| `escape-others` | `char` | `[1, 12]` | '\-<>#/ ,:=!@' |
| `escape-whitespace` | `char` | `[1, 5]` | ['a	b' char(10) 'c'] |
| `wildcard-star` | `char` | `[1, 5]` | '.*\.m' |
| `wildcard-question` | `char` | `[1, 3]` | 'a.c' |
| `wildcard-dot` | `char` | `[1, 4]` | 'a\.b' |
| `wildcard-use` | `logical` | `[1, 1]` | true |
| `string-input` | `string` | `[1, 1]` | "a\.b" |
| `cell-input` | `cell` | `[1, 2]` | ["'a\\.b'", "'c\\+d'"] |
| `empty` | `char` | `[0, 0]` | '' |
| `err-op` | — | — | error `MATLAB:REGEXP:invalidOptions` |

## Unresolved questions

- `regexptranslate.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
- `regexptranslate.q-other-ops` — Operations other than 'escape' and 'wildcard' were not exercised.
