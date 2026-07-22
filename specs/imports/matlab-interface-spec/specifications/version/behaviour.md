# `version` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/version_observations.json`](observations/version_observations.json)

## Summary

Return text identifying the MATLAB version; options request substrings such as the release or build date.

## Confirmed invocation forms

- `v = version` — exercised in the experiment (case `default`).
- `v = version(option)` — exercised in the experiment.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `char` for the observed inputs. *(cases: `default`, `release`)*
2. **Confirmed** — Returns a char row vector; version('-release') returns the release substring (observed '26.1.0.3276743 (R2026a) Update 3' and '2026a' on this installation). The specific strings are release-dependent. *(cases: `default`, `release`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `default` | `char` | `[1, 32]` | `'26.1.0.3276743 (R2026a) Update 3'` |
| `release` | `char` | `[1, 5]` | `'2026a'` |

## Unresolved questions

- `version.q-values` — The specific strings are release-dependent; record only that char text is returned.
