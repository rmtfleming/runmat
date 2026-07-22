# `strtok` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/strtok_observations.json`](observations/strtok_observations.json)

## Summary

Return the first token of a string, using whitespace by default or a supplied set of delimiters; a second output returns the remainder.

## Confirmed invocation form

- `token = strtok(str)` — exercised in the experiment (case `default`).

Additional forms are proposed but not yet exercised:
- `[token, remain] = strtok(str, delimiters)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `char` for the observed inputs. *(cases: `default`, `custom-delim`, `leading-space`)*
2. **Confirmed** — The first output is the leading token; the default delimiter set is whitespace, a delimiter set may be supplied, and leading delimiters are skipped. *(cases: `default`, `custom-delim`, `leading-space`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `default` | `char` | `[1, 5]` | `'hello'` |
| `custom-delim` | `char` | `[1, 1]` | `'a'` |
| `leading-space` | `char` | `[1, 4]` | `'lead'` |

## Unresolved questions

- `strtok.q-remainder` — Specify the second output (remain), including leading delimiter handling.
- `strtok.q-empty` — Behaviour on empty input or input consisting only of delimiters.
