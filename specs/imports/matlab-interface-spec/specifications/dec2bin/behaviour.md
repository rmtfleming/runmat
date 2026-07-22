# `dec2bin` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/dec2bin_observations.json`](observations/dec2bin_observations.json)

## Summary

Convert nonnegative integers to their binary representation as a character array, optionally zero-padded to a minimum length.

## Confirmed invocation forms

- `str = dec2bin(D)` — exercised in the experiment (case `scalar`).
- `str = dec2bin(D, n)` — exercised in the experiment.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `char` for the observed inputs. *(cases: `scalar`, `padded`, `zero`, `width-below-natural`)*
2. **Confirmed** — Returns a char array of binary digits (5 -> '101'); a width argument zero-pads up to at least that many digits and does not truncate when the requested width is below the natural width (5, 2 -> '101'); zero gives '0'. *(cases: `scalar`, `padded`, `zero`, `width-below-natural`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `scalar` | `char` | `[1, 3]` | `'101'` |
| `padded` | `char` | `[1, 8]` | `'00000101'` |
| `zero` | `char` | `[1, 1]` | `'0'` |
| `width-below-natural` | `char` | `[1, 3]` | `'101'` |

## Unresolved questions

- `dec2bin.q-vector` — Output layout for a vector of inputs (char matrix).
- `dec2bin.q-negative` — Behaviour for negative or non-integer inputs.
