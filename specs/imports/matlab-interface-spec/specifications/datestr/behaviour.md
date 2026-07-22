# `datestr` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/datestr_observations.json`](observations/datestr_observations.json)

## Summary

Convert a date/time value (serial date number or text) to a character representation, optionally using a specified format. (Documented as not recommended / legacy; verify by observation.)

## Confirmed invocation form

- `s = datestr(t)` — exercised in the experiment (case `serial-default`).

Additional forms are proposed but not yet exercised:
- `s = datestr(t, formatOut)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `char` for the observed inputs. *(cases: `serial-default`, `serial-format`)*
2. **Confirmed** — Returns a char row vector; the default format is day-month-year (for example 30-Dec-2022) and a format string overrides it (yyyy-mm-dd). *(cases: `serial-default`, `serial-format`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `serial-default` | `char` | `[1, 11]` | `'30-Dec-2022'` |
| `serial-format` | `char` | `[1, 10]` | `'2022-12-30'` |

## Unresolved questions

- `datestr.q-deprecated` — datestr is documented as not recommended (string(datetime) is suggested); record any notice.
- `datestr.q-locale` — Record locale/format dependence of the default output.
