# `display` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/display_observations.json`](observations/display_observations.json)

## Summary

Display a value in the command window (the method invoked automatically when a statement is not terminated by a semicolon).

## Confirmed invocation forms

- `display(X)` — exercised in the experiment (case `scalar-text`).

Additional forms are proposed but not yet exercised:
- `display(X, name)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The text captured from the command window with evalc uses leading whitespace and trailing newlines, and a char array is shown without surrounding quotes. (These cases observe evalc-captured output, not display's own return value.) *(cases: `scalar-text`, `vector-text`, `char-text`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `scalar-text` | `char` | `[1, 8]` | `['    42' char(10) '' char(10) '']` |
| `vector-text` | `char` | `[1, 20]` | `['     1     2     3' char(10) '' char(10) '']` |
| `char-text` | `char` | `[1, 3]` | `['hi' char(10) '']` |

## Unresolved questions

- `display.q-side-effect` — display's primary effect is command-window output, and capturing that text interacts with formatting; scope how to specify it.
- `display.q-overload` — display is overloadable per class; scope which classes to specify.
- `display.q-return` — display's return value (if any) was not directly observed; the recorded char is evalc-captured command-window text, not display's return.
