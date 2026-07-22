# `int2str` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/int2str_observations.json`](observations/int2str_observations.json)

## Summary

Round the input to the nearest integers and convert the result to a character array.

## Confirmed invocation form

- `str = int2str(X)` — exercised in the experiment (case `scalar`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `char` for the observed inputs. *(cases: `scalar`, `negative`, `vector`, `matrix`)*
2. **Confirmed** — Rounds to the nearest integer with ties away from zero (observed -2.5 -> -3, 5.5 -> 6) and returns a char array; multi-element inputs are space-separated. *(cases: `scalar`, `negative`, `vector`, `matrix`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `scalar` | `char` | `[1, 1]` | `'4'` |
| `negative` | `char` | `[1, 2]` | `'-3'` |
| `vector` | `char` | `[1, 7]` | `'1  4  6'` |
| `matrix` | `char` | `[2, 4]` | `['1  3';'4  4']` |

## Unresolved questions

- `int2str.q-rounding` — Confirm the rounding rule (e.g. round half away from zero) used before conversion.
- `int2str.q-spacing` — Confirm column spacing/alignment of the character output.
