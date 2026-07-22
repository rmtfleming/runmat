# `matches` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/matches_observations.json`](observations/matches_observations.json)

## Summary

Return a logical array indicating which elements of the input equal a given pattern or set of patterns.

## Confirmed invocation form

- `tf = matches(str, pat)` — exercised in the experiment (case `in-set`).

Additional forms are proposed but not yet exercised:
- `tf = matches(str, pat, 'IgnoreCase', true)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `logical` for the observed inputs. *(cases: `in-set`, `scalar`, `ignore-case`)*
2. **Confirmed** — Returns a logical; a scalar first argument yields a scalar result that is true when it equals any of the given patterns, and the 'IgnoreCase' option enables case-insensitive matching. *(cases: `in-set`, `scalar`, `ignore-case`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `in-set` | `logical` | `[1, 1]` | `true` |
| `scalar` | `logical` | `[1, 1]` | `true` |
| `ignore-case` | `logical` | `[1, 1]` | `true` |

## Unresolved questions

- `matches.q-release` — matches was introduced in a specific release; record applicable releases.
- `matches.q-string-vs-char` — Confirm behaviour differences between string and char inputs.
