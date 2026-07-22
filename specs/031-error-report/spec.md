# Feature Specification: Exception Report — `getReport`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`031-error-report`; export `SPECIFY_FEATURE=031-error-report`)

**Created**: 2026-07-22

**Status**: Tier B batch 3 — implementation authorised (F31 gate decision
recorded 2026-07-22 in
`specs/001-clean-room-matlab-builtins/selection-proposal-tierb3.md`,
"F29–F32 authorised").

**Input**: Approved Tier B export `getReport` 0.2.0 (see `provenance.md`;
import pinned at source commit `1b019121bb747e63d69a3eb005534cb60669e5f4`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read an exception's report text (Priority: P1)

RunMat users calling `getReport(ME)` on an `MException` get a char report
describing the exception. For a plain exception the report is the exception's
message text. On the observed case `getReport(MException('x:y','oops'))`
returns `'oops'`, a 1×4 char row vector.

**Independent Test**: run the observed case from the export through the
builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** an `MException` with identifier `'x:y'` and message `'oops'`,
   **When** `getReport(ME)` is called, **Then** the result is `'oops'`, class
   char, size 1×4.
   [getReport.signature-primary, getReport.char-report, getReport.output-class]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-031-01** `getReport` MUST accept the primary invocation form
  `report = getReport(ME)` [getReport.signature-primary] and return class char
  [getReport.output-class]. For the observed input
  `MException('x:y','oops')` the report MUST be `'oops'` (1×4)
  [getReport.char-report].
- **FR-031-02** The output MUST be a char row vector holding the exception's
  message text; its length tracks the message
  [getReport.char-report, getReport.output-class].
- **FR-031-03** The report content is the `MException` message text alone.
  Identifier formatting and stack-trace rendering are NOT observed
  (`getReport.q-options`); by documented independent choice the report is the
  message text for every `MException`, including those with a populated stack,
  and an empty message yields an empty (1×0) char row. These extended
  behaviours are NOT asserted MATLAB-conformant.
- **FR-031-04** The input-type error path is unobserved. By documented
  independent choice a non-`MException` input raises
  `RunMat:getReport:InvalidInput`.
- **FR-031-05** The `'extended'`/`'basic'` and hyperlink options are
  unresolved in the approved export (`getReport.q-options`, not exercised) and
  MUST be omitted: the builtin declares a Fixed single output and a single
  one-argument signature.

### Key Entities

- Input: `Value::MException` (RunMat's exception value kind, with `identifier`,
  `message`, and `stack` fields).
- Result: `Value::CharArray` row vector (RunMat's char row vector — the
  observed class of the report).

## Success Criteria *(mandatory)*

- **SC-031-1**: The observed case reproduces exactly (value `'oops'`, class
  char, size 1×4) in a normative test.
- **SC-031-2**: `getReport` is registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime` passes for its
  test module.
- **SC-031-3**: Traceability: each FR cites ≥1 claim id (or is explicitly a
  documented choice); each normative test cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `getReport.q-options`: the `'extended'`/`'basic'` format selectors and
  hyperlink handling are not exercised in the export. Choice: omit them
  (Fixed single output, single one-argument signature); revisit when the
  specification side observes them.
- Stack-trace / identifier formatting: only the message-only report of a
  plain `MException` is observed. Choice: return the message text for every
  `MException` (message-only), including ones carrying a stack, until the
  specification side observes the fuller multi-line report.
- Empty message: unobserved. Choice: return an empty (1×0) char row.
- Input-type error: the rejection of a non-`MException` argument is
  unobserved. Choice: raise `RunMat:getReport:InvalidInput` with a stable
  identifier.

## Assumptions

- RunMat's value model already provides `Value::MException` (with a `message`
  field) and `CharArray` row vectors; no new runtime types are required.
- The observed report equals the exception's message text, so RunMat's
  `MException.message` is the sole source for the report.
