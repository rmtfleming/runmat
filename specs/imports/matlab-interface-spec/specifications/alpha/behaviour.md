# `alpha` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/alpha_observations.json`](observations/alpha_observations.json)

## Summary

Set the face transparency of the surfaces and patches in the current axes, or of a given object; returns no output.

## Confirmed invocation forms

- `alpha(value)` — Set FaceAlpha of surfaces/patches in the current axes. *(e.g. case `scalar`)*
- `alpha(obj, value)` — Set FaceAlpha of `obj`. *(e.g. case `object`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — A numeric value sets FaceAlpha of a surface (0.5) or patch (0.4) in the current axes; alpha(obj, 0.3) sets it on the given object. *(cases: `scalar`, `patch`, `object`)*
2. **Confirmed** — 'clear' sets FaceAlpha 0, 'opaque' sets 1, and 'flat' and 'interp' set FaceAlpha to those keywords. *(cases: `clear`, `opaque`, `flat`, `interp`)*
3. **Confirmed** — EdgeAlpha stayed 1 after alpha(0.5). *(cases: `edge-untouched`)*
4. **Confirmed** — With only a line in the axes, alpha(0.5) completes without error. *(cases: `line-only`)*
5. **Confirmed** — Requesting an output errors. *(cases: `output-arg`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| An unrecognised keyword is given. | `MATLAB:alpha:UnknownAlphaValue` | `err-bad-keyword` |
| An output argument is requested. | `MATLAB:TooManyOutputs` | `output-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `scalar` | `double` | `[1, 1]` | 0.5 |
| `object` | `double` | `[1, 1]` | 0.29999999999999999 |
| `clear` | `double` | `[1, 1]` | 0 |
| `opaque` | `double` | `[1, 1]` | 1 |
| `flat` | `char` | `[1, 4]` | 'flat' |
| `interp` | `char` | `[1, 6]` | 'interp' |
| `patch` | `double` | `[1, 1]` | 0.40000000000000002 |
| `edge-untouched` | `double` | `[1, 1]` | 1 |
| `line-only` | `logical` | `[1, 1]` | true |
| `output-arg` | — | — | error `MATLAB:TooManyOutputs` |
| `err-bad-keyword` | — | — | error `MATLAB:alpha:UnknownAlphaValue` |

## Unresolved questions

- `alpha.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `alpha.q-more-forms` — Forms with more arguments, numeric values outside those observed, and other object types were not exercised.
