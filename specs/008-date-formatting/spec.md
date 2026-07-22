# Feature Specification: Date Formatting — `datestr`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `008-date-formatting`; export `SPECIFY_FEATURE=008-date-formatting`)

**Created**: 2026-07-22

**Status**: Batch 2 ("all batches" run, 2026-07-22) — implemented under
blanket gate approval; publication excluded

**Input**: Approved Tier A export `datestr` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Convert a serial date number to date text (Priority: P1)

RunMat users calling `datestr(t)` on a serial date number get a char row
vector in the default day-month-year format, matching the approved observed
behaviour exactly.

**Independent Test**: run each observed case from the export through the
builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** serial date number `738885` (case `serial-default`), **When**
   `datestr(738885)` is called, **Then** the result is char `'30-Dec-2022'`
   with size 1×11. [datestr.signature-primary, datestr.output-class,
   datestr.char-format]
2. **Given** the same serial number and format `'yyyy-mm-dd'` (case
   `serial-format`), **When** `datestr(738885, 'yyyy-mm-dd')` is called,
   **Then** the result is char `'2022-12-30'` with size 1×10.
   [datestr.output-class, datestr.char-format]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-008-01** `datestr` MUST accept the primary form `s = datestr(t)` for
  a scalar numeric serial date number [datestr.signature-primary] and return
  class char [datestr.output-class].
- **FR-008-02** The default output format MUST be day-month-year
  (`dd-mmm-yyyy` tokens), reproducing observed case `serial-default`:
  `datestr(738885)` => `'30-Dec-2022'`, char 1×11 [datestr.char-format].
- **FR-008-03** A format argument MUST override the default, reproducing
  observed case `serial-format`: `datestr(738885, 'yyyy-mm-dd')` =>
  `'2022-12-30'`, char 1×10 [datestr.char-format].
- **FR-008-04** The output MUST be a char row vector (1×N)
  [datestr.output-class, datestr.char-format].
- **FR-008-05** Inputs and format tokens not covered by the export (date
  text, date vectors, non-scalar arrays, fractional serial numbers,
  time-of-day tokens, unknown tokens, locale variation, GPU values) MUST be
  handled by documented independent choice (see Unresolved behaviour) and
  MUST NOT be asserted as MATLAB-conformant.

### Key Entities

- Result: `Value::CharArray` row vector (RunMat's char class).
- Date conversion: an independently implemented civil-from-days algorithm
  (publicly documented, public-domain Hinnant algorithm) with the serial-day
  epoch anchored so observed case `serial-default` reproduces exactly
  (738885 → 30-Dec-2022).

## Success Criteria *(mandatory)*

- **SC-008-1**: Both observed cases in the export reproduce exactly (value,
  class char, sizes 1×11 and 1×10) in normative tests.
- **SC-008-2**: `datestr` registered and discoverable through the standard
  `#[runtime_builtin]` registry; `cargo test -p runmat-runtime` passes for
  its test module.
- **SC-008-3**: Traceability: each FR cites ≥1 claim id; each normative test
  cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `datestr.q-deprecated` (carried): datestr is documented upstream as
  legacy/not recommended. Choice: RunMat emits no deprecation notice;
  question returned to the specification side.
- `datestr.q-locale` (carried): locale dependence of the default output is
  unobserved. Choice: `mmm` renders English three-letter month abbreviations
  Jan..Dec.
- Format token set: only `yyyy`, `mm`, `dd` (observed via `serial-format`),
  `mmm` (exercised by the default format), and — as an unobserved documented
  choice — `HH`, `MM`, `SS` (hour/minute/second of day) are supported. Any
  other alphabetic character in a format is rejected with a clear error
  (`RunMat:datestr:InvalidFormat`) rather than guessed.
- Fractional serial numbers (time of day): unobserved. Choice: the
  fractional day is a time of day, rounded to the nearest second, carrying
  across midnight when rounding reaches 24:00:00.
- Epoch anchoring: chosen so `738885` → 30-Dec-2022 (observed); a
  consequence is serial day 1 = 01-Jan-0000 and 719529 = 01-Jan-1970 in the
  proleptic Gregorian calendar (both unobserved, tested as
  `unresolved_choice`).
- Input domain: date text and date vectors are `pending` in the approved
  interface with no observations. Choice: only finite scalar numeric serial
  date numbers (including 1×1 tensors) are accepted; everything else errors
  with `RunMat:datestr:InvalidInput`.
- String-scalar format arguments: unobserved. Choice: accepted as format
  text; the output stays char per `datestr.output-class`.

## Assumptions

- No new runtime types required; the result uses RunMat's existing
  `CharArray`.
- GPU inputs are gathered immediately (scalar metadata-style builtin; no
  device kernel).
