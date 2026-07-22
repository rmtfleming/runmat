# `system` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/system_observations.json`](observations/system_observations.json)

## Summary

Run an operating-system command from within MATLAB and return its exit status and captured output.

## Confirmed invocation forms

- `status = system(command)` — exercised in the experiment (case `status-success`).
- `[status, cmdout] = system(command)` — exercised in the experiment.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Returns the command exit status as a numeric scalar (0 for a successful command; the failing command tested, false, returned 1; the full range of nonzero failure codes was not explored). The second output captures standard output as a char row vector including the trailing newline (echo hello yielded 'hello' followed by a newline). *(cases: `status-success`, `status-failure`, `captured-output`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `status-success` | `double` | `[1, 1]` | `0` |
| `status-failure` | `double` | `[1, 1]` | `1` |
| `captured-output` | `char` | `[1, 6]` | `['hello' char(10) '']` |

## Unresolved questions

- `system.q-side-effect` — system spawns an external process; experiments must use benign, platform-portable commands and record platform dependence.
- `system.q-outputs` — The text encoding of cmdout and the range of nonzero exit codes across platforms were not explored.
