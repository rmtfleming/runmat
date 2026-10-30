# `date` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/date_observations.json`](observations/date_observations.json)

## Summary

Return the current date as a 1x11 char vector: day, hyphen, three-letter month, hyphen, four-digit year (observed on one date).

## Confirmed invocation forms

- `d = date` — Current date as text. *(e.g. case `class-shape`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is a 1x11 char vector. *(cases: `class-shape`)*
2. **Confirmed** — The text matched the pattern ^\d{2}-[A-Z][a-z]{2}-\d{4}$ and the day field had two characters on the run date. *(cases: `pattern`, `leading-zero`)*
3. **Confirmed** — The day, month and year fields agreed with clock; on the run date (October) the month field was 'Oct'; other months were not observed. *(cases: `agrees-clock`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| An input argument is given. | `MATLAB:TooManyInputs` | `err-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `class-shape` | `struct` | `[1, 1]` | class='char'; size=[1 11] |
| `pattern` | `logical` | `[1, 1]` | true |
| `agrees-clock` | `struct` | `[1, 1]` | day_ok=true; month_ok=true; year_ok=true |
| `leading-zero` | `logical` | `[1, 1]` | true |
| `err-arg` | — | — | error `MATLAB:TooManyInputs` |

## Unresolved questions

- `date.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `date.q-zero-pad` — Whether a day below 10 is zero-padded is not established: only day 30 was observed.
- `date.q-other-months` — Month text was observed only for October.
- `date.q-locale` — Only one locale was exercised; whether the month text depends on locale is not established.
