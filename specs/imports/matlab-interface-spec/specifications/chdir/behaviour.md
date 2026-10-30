# `chdir` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/chdir_observations.json`](observations/chdir_observations.json)

## Summary

Change the current folder; with an output, return the folder that was current before the call.

## Confirmed invocation forms

- `chdir(folder)` — Make `folder` the current folder. *(e.g. case `changes`)*
- `old = chdir(folder)` — Change folder and return the previous current folder. *(e.g. case `output-old`)*
- `d = chdir` — Return the current folder without changing it. *(e.g. case `no-arg-output`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — After chdir(tempdir) the current folder (pwd) is the temp folder; a string-scalar target behaves the same. *(cases: `changes`, `string`)*
2. **Confirmed** — The target with and without a trailing file separator led to the same folder. *(cases: `trailing-sep`)*
3. **Confirmed** — chdir('..') from the temp folder moved to its parent folder. *(cases: `relative`)*
4. **Confirmed** — old = chdir(folder) returned a 1-by-n char row equal to the folder that was current before the call (which was not the temp folder), and still changed folder. *(cases: `output-old`)*
5. **Confirmed** — d = chdir with no input returned a 1-by-n char row equal to pwd. *(cases: `no-arg-output`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The folder does not exist. | `MATLAB:cd:NonExistentFolder` | `err-missing` |
| The folder argument is numeric. | `MATLAB:string` | `err-numeric` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `changes` | `logical` | `[1, 1]` | true |
| `trailing-sep` | `struct` | `[1, 1]` | same_folder=true; is_tempdir=true |
| `output-old` | `struct` | `[1, 1]` | class='char'; size=[1 74]; start_was_tempdir=false; equals_previous=true; now_tempdir=true |
| `no-arg-output` | `struct` | `[1, 1]` | class='char'; size=[1 74]; equals_pwd=true |
| `relative` | `logical` | `[1, 1]` | true |
| `string` | `logical` | `[1, 1]` | true |
| `err-missing` | — | — | error `MATLAB:cd:NonExistentFolder` |
| `err-numeric` | — | — | error `MATLAB:string` |

## Unresolved questions

- `chdir.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `chdir.q-display` — The no-input, no-output form (display of the current folder) was not exercised.
