# Feature Specification: Numeric/String Conversion — `int2str`, `str2num`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`007-numeric-string-conversion`; export
`SPECIFY_FEATURE=007-numeric-string-conversion`)

**Created**: 2026-07-22

**Status**: Implemented (Batch review pending)

**Input**: Approved Tier A exports `int2str` 0.2.0, `str2num` 0.2.0 (see
`provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Format numeric values as integer text (Priority: P1)

RunMat users calling `int2str(X)` get a char array holding each element of
`X` rounded to the nearest integer (ties away from zero), matching the
approved observed behaviour.

**Independent Test**: run each observed case from the export through the
builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** the scalar `3.7`, **When** `int2str` is called, **Then** the
   result is the 1×1 char `'4'`. [int2str.rounding-and-char,
   int2str.output-class]
2. **Given** the scalar `-2.5`, **When** `int2str` is called, **Then** the
   result is the 1×2 char `'-3'` — the tie rounds away from zero.
   [int2str.rounding-and-char]
3. **Given** the row vector `[1.2 3.8 5.5]`, **When** `int2str` is called,
   **Then** the result is the 1×7 char `'1  4  6'` — elements joined with
   two spaces. [int2str.rounding-and-char]
4. **Given** the matrix `[1.1 2.9; 3.5 4.4]`, **When** `int2str` is called,
   **Then** the result is the 2×4 char `['1  3';'4  4']` — output rows
   mirror input rows. [int2str.rounding-and-char, int2str.output-class]

### User Story 2 - Convert numeric text to values (Priority: P1)

RunMat users calling `str2num(chr)` get double values parsed from the text,
including bracketed vectors/matrices and evaluated arithmetic expressions.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** `'42'`, **When** `str2num` is called, **Then** the result is
   the 1×1 double `42`. [str2num.parse-eval, str2num.output-class]
2. **Given** `'[1 2 3]'`, **When** `str2num` is called, **Then** the result
   is the 1×3 double `[1 2 3]`. [str2num.parse-eval]
3. **Given** `'[1 2; 3 4]'`, **When** `str2num` is called, **Then** the
   result is the 2×2 double `[1 2;3 4]`. [str2num.parse-eval]
4. **Given** `'2+3'`, **When** `str2num` is called, **Then** the result is
   the 1×1 double `5` — expressions evaluate. [str2num.parse-eval]
5. **Given** text that cannot be parsed, **When** `str2num` is called,
   **Then** the result is empty. *(Summary-derived: approved spec summary
   text; no backing observation case — see Unresolved. Tests tagged
   `summary_derived`.)*

## Requirements *(mandatory)*

### Functional Requirements

- **FR-007-01** `int2str` MUST accept one argument (`str = int2str(X)`)
  [int2str.signature-primary] and return class char
  [int2str.output-class].
- **FR-007-02** `int2str` MUST round each element to the nearest integer
  with ties away from zero (observed `-2.5 -> -3`, `5.5 -> 6`) before
  conversion, reproducing all observed values exactly
  [int2str.rounding-and-char].
- **FR-007-03** Multi-element `int2str` output MUST be space-separated with
  the observed layout: elements joined by two spaces (`'1  4  6'` is 1×7),
  and the char matrix rows MUST equal the input rows (2×2 input → 2×4
  char) [int2str.rounding-and-char]. Alignment for column widths beyond the
  observed single-digit cases is an independent choice (see Unresolved).
- **FR-007-04** `str2num` MUST accept one argument (`X = str2num(chr)`)
  [str2num.signature-primary] and return class double
  [str2num.output-class].
- **FR-007-05** `str2num` MUST parse numeric literals, bracketed row
  vectors, and bracketed matrices to the observed values/shapes, and MUST
  evaluate arithmetic expressions (observed `'2+3' -> 5`)
  [str2num.parse-eval]. Standard operator precedence/grouping beyond the
  observed case is summary-derived.
- **FR-007-06** (summary-derived) `str2num` SHOULD return empty when the
  input cannot be parsed, per the approved summary; no observation case
  backs the concrete class/shape (implemented as 0×0 double — documented
  choice under `str2num.q-eval`).
- **FR-007-07** Inputs and behaviours not covered by the exports (complex,
  NaN/Inf, empty, non-text `str2num` input, workspace-variable expressions,
  the two-output form, …) MUST be handled by documented independent choice
  (see Unresolved behaviour) and MUST NOT be asserted as
  MATLAB-conformant.

### Key Entities

- `int2str` result: `Value::CharArray` (RunMat's char matrix).
- `str2num` result: `Value::Num` for 1×1 doubles, `Value::Tensor`
  otherwise; empty is a 0×0 `Value::Tensor`.
- `str2num` evaluation: a self-contained numeric parser/evaluator covers
  the observed grammar; text outside it is delegated to RunMat's registered
  `eval` builtin (`builtins/introspection/dynamic_workspace.rs`), never to
  MATLAB. RunMat's `eval` executes only inside a VM workspace frame; outside
  one the delegation fails and maps to the documented empty result.

## Success Criteria *(mandatory)*

- **SC-007-1**: Every observed case in the two exports reproduces exactly
  (value, class, size) in normative tests.
- **SC-007-2**: Both builtins registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime` passes for
  their test modules.
- **SC-007-3**: Traceability: each FR cites ≥1 claim id; each normative
  test cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `int2str.q-rounding`: rounding rule confirmation. The observed ties
  (`-2.5 -> -3`, `5.5 -> 6`) fix ties-away-from-zero for the observed
  values; other tie-breaking corners remain to be confirmed spec-side.
- `int2str.q-spacing`: column spacing/alignment beyond single-digit
  columns. Choice: right-aligned columns joined by two spaces, all rows
  equal width (consistent with the observed 1×7 and 2×4 extents).
- `int2str` on NaN/Inf, empty, integer-typed, logical, complex, or
  non-numeric inputs: unobserved. Choices: `NaN`/`Inf`/`-Inf` word
  spellings; empty input → 0×0 char; integer/logical treated numerically;
  complex and non-numeric inputs rejected
  (`RunMat:int2str:InvalidInput`).
- `str2num.q-eval`: observable failure mode for unparsable input. Choice:
  0×0 empty double for every failed parse/evaluation. Security note: the
  self-contained parser evaluates numeric expressions only; full-language
  evaluation is reachable solely through RunMat's VM-gated `eval` builtin
  (subject to the host's dynamic-eval policy).
- `str2num.q-second-output`: the `[X, ok]` two-output form is unspecified
  in the export and is not implemented.
- `str2num` tokenisation beyond space-separated bracketed literals
  (commas, attached signs, nested brackets): unobserved. Choices: commas
  separate elements; an attached sign (`'[1 -2]'`) starts a new element
  while a spaced operator (`'[1 - 2]'`) continues the expression; nested
  concatenation falls through to `eval` delegation (today: empty).
- `str2num` non-text input: unobserved. Choice: rejected
  (`RunMat:str2num:InvalidInput`); string scalars are accepted as text.

## Assumptions

- RunMat value model already provides `Value::CharArray`, `Value::Num`,
  `Value::Tensor` (column-major); no new runtime types required.
- Rust's `f64::round` (ties away from zero) matches the observed rounding
  of the export's cases; no bespoke rounding code is needed.
- Delegating unparsable text to the registered `eval` builtin keeps
  `str2num` forward-compatible with full-language evaluation without
  duplicating the interpreter inside the runtime crate.
