# `computer` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/computer_observations.json`](observations/computer_observations.json)

## Summary

Return text identifying the computer type MATLAB is running on; an option requests the short architecture string.

## Confirmed invocation forms

- `str = computer` — exercised in the experiment (case `default`).
- `str = computer('arch')` — exercised in the experiment.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `char` for the observed inputs. *(cases: `default`, `arch`)*
2. **Confirmed** — Returns a char row vector; the default form gives the platform identifier in upper case (GLNXA64 on this platform) and the 'arch' option gives the lower-case architecture string (glnxa64). *(cases: `default`, `arch`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `default` | `char` | `[1, 7]` | `'GLNXA64'` |
| `arch` | `char` | `[1, 7]` | `'glnxa64'` |

## Unresolved questions

- `computer.q-multi-output` — Specify the [str, maxsize, endian] multi-output form (not exercised).
