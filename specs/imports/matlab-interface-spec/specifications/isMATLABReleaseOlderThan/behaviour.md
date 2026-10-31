# `isMATLABReleaseOlderThan` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isMATLABReleaseOlderThan_observations.json`](observations/isMATLABReleaseOlderThan_observations.json)

## Summary

Return true if the running release is older than the named release; observed on R2026a.

## Confirmed invocation forms

- `tf = isMATLABReleaseOlderThan(release)` — Compare the running release with `release` (COBRA form). *(e.g. case `older`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — On R2026a: 'R2022a' and 'R2006a' gave false; the running release 'R2026a' gave false; 'R2026b', 'R2027a' and 'R2030a' gave true. *(cases: `older`, `much-older`, `same`, `later-b`, `string`, `later-year`)*
2. **Confirmed** — A string scalar is accepted, and a lower-case name 'r2022a' was accepted (false). *(cases: `string`, `lowercase`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The text is not a release name ('2022a'). | `MATLAB:isMATLABReleaseOlderThan:invalidReleaseInput` | `err-bad-name` |
| The input is numeric. | `MATLAB:validators:mustBeTextScalar` | `err-numeric` |
| No input is given. | `MATLAB:minrhs` | `err-none` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `older` | `logical` | `[1, 1]` | false |
| `same` | `logical` | `[1, 1]` | false |
| `later-b` | `logical` | `[1, 1]` | true |
| `later-year` | `logical` | `[1, 1]` | true |
| `much-older` | `logical` | `[1, 1]` | false |
| `string` | `logical` | `[1, 1]` | true |
| `lowercase` | `logical` | `[1, 1]` | false |
| `err-bad-name` | — | — | error `MATLAB:isMATLABReleaseOlderThan:invalidReleaseInput` |
| `err-numeric` | — | — | error `MATLAB:validators:mustBeTextScalar` |
| `err-none` | — | — | error `MATLAB:minrhs` |

## Unresolved questions

- `isMATLABReleaseOlderThan.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `isMATLABReleaseOlderThan.q-release` — Results depend on the running release; only R2026a was exercised. Calls with more than one argument were not exercised.
