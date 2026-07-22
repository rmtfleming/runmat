# `append` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/append_observations.json`](observations/append_observations.json)

## Summary

Append text pieces together, returning the combined text.

## Confirmed invocation forms

- `str = append(str1, str2, ...)` — exercised in the experiment (case `strings`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Concatenates the text arguments; string inputs yield a string scalar ("a", "b" gave "ab") and char inputs yield a char array ('a', 'b' gave 'ab'). *(cases: `strings`, `multiple`, `char`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `strings` | `string` | `[1, 1]` | `<string [1 1]>` |
| `multiple` | `string` | `[1, 1]` | `<string [1 1]>` |
| `char` | `char` | `[1, 2]` | `'ab'` |

## Unresolved questions

- `append.q-release` — append was introduced in a specific release; record applicable releases.
- `append.q-types` — Output type for char vs string vs mixed inputs.
