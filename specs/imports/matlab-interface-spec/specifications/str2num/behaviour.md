# `str2num` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/str2num_observations.json`](observations/str2num_observations.json)

## Summary

Convert a character representation of a numeric array to numeric values by evaluating it; returns empty when the input cannot be parsed.

## Confirmed invocation form

- `X = str2num(chr)` — exercised in the experiment (case `scalar`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `scalar`, `row-vector`, `matrix`, `expression`)*
2. **Confirmed** — Parses the text to a numeric array and evaluates arithmetic expressions (the text '2+3' produced 5); bracketed text produced row vectors and matrices. *(cases: `scalar`, `row-vector`, `matrix`, `expression`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `scalar` | `double` | `[1, 1]` | `42` |
| `row-vector` | `double` | `[1, 3]` | `[1 2 3]` |
| `matrix` | `double` | `[2, 2]` | `[1 2;3 4]` |
| `expression` | `double` | `[1, 1]` | `5` |

## Unresolved questions

- `str2num.q-eval` — str2num evaluates its input; record the observable failure mode (empty output) for unparsable input and any security-relevant notes.
- `str2num.q-second-output` — Specify the two-output form [X, ok] if present.
