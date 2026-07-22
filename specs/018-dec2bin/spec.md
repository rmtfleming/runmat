# Feature Specification: Binary Conversion — `dec2bin`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `018-dec2bin`; export `SPECIFY_FEATURE=018-dec2bin`)

**Created**: 2026-07-22

**Status**: Gate-approved (Tier B batch 1); implementation complete

**Input**: Approved Tier B export `dec2bin` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Convert a nonnegative integer to binary text (Priority: P1)

RunMat users calling `dec2bin(D)` get a char row of binary digits for a
nonnegative integer `D`, with no leading zeros beyond the natural width;
`dec2bin(D, n)` zero-pads on the left to at least `n` digits and never
truncates below the natural width.

**Independent Test**: run each observed case from the export through the
builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** the scalar 5, **When** `dec2bin(5)` is called, **Then** the
   result is `'101'`, class char, size 1×3. [dec2bin.signature-primary,
   dec2bin.output-class, dec2bin.binary-char]
2. **Given** the scalar 5 and width 8, **When** `dec2bin(5, 8)` is called,
   **Then** the result is `'00000101'`, class char, size 1×8 (left
   zero-padding to at least the requested width). [dec2bin.binary-char,
   dec2bin.output-class]
3. **Given** the scalar 0, **When** `dec2bin(0)` is called, **Then** the
   result is `'0'`, class char, size 1×1. [dec2bin.binary-char]
4. **Given** the scalar 5 and width 2 (below the natural width 3), **When**
   `dec2bin(5, 2)` is called, **Then** the result is `'101'`, class char,
   size 1×3 — the width argument never truncates. [dec2bin.binary-char]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-018-01** `dec2bin` MUST accept one argument (`str = dec2bin(D)`)
  [dec2bin.signature-primary] and an optional second width argument
  (`str = dec2bin(D, n)`).
- **FR-018-02** The output MUST have class char for the observed inputs
  [dec2bin.output-class].
- **FR-018-03** A nonnegative integer scalar MUST convert to its binary digit
  text with no leading zeros beyond the natural width (`5` → `'101'`, 1×3;
  `0` → `'0'`, 1×1) [dec2bin.binary-char].
- **FR-018-04** The width argument MUST zero-pad on the left so the output
  has at least `n` digits (`dec2bin(5, 8)` → `'00000101'`, 1×8)
  [dec2bin.binary-char].
- **FR-018-05** A width below the natural width MUST NOT truncate the output
  (`dec2bin(5, 2)` → `'101'`, 1×3) [dec2bin.binary-char].
- **FR-018-06** Inputs not covered by the export (vectors/matrices, negative
  or non-integer values, non-finite values, empty arrays, logical/integer
  classes, invalid widths, GPU residency) MUST be handled by documented
  independent choice (see Unresolved behaviour) and MUST NOT be asserted as
  MATLAB-conformant.

### Key Entities

- Conversion result: `Value::CharArray` (RunMat's char matrix; a scalar input
  yields one 1×w row).
- Input elements: RunMat numeric/logical values, gathered from the GPU
  before formatting (residency policy GatherImmediately).

## Success Criteria *(mandatory)*

- **SC-018-1**: Every observed case in the export reproduces exactly (value,
  class char, size) in normative tests.
- **SC-018-2**: `dec2bin` is registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime` passes for its
  test module.
- **SC-018-3**: Traceability: each FR cites ≥1 claim id; each normative test
  cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `dec2bin.q-vector`: output layout for a vector of inputs is unobserved.
  Choice: one row per element (column-major element order), all rows
  zero-padded to a common width of max(requested width, widest natural
  width); empty input yields a 0×0 char array.
- `dec2bin.q-negative`: behaviour for negative or non-integer inputs is
  unobserved. Choice: non-integer values round to the nearest integer with
  ties away from zero; negative and non-finite values error with identifier
  `RunMat:dec2bin:InvalidInput`; values above 2^53 (not exactly
  representable in double) also error.
- Width argument classes are unobserved beyond integer scalars 8 and 2.
  Choice: the width must be a finite nonnegative real numeric scalar
  (non-integer widths round to nearest); anything else, or more than two
  arguments, errors with identifier `RunMat:dec2bin:InvalidWidth`.
- Logical and integer-typed inputs are unobserved. Choice: converted via
  their numeric values.

## Assumptions

- RunMat's `Value` model already provides `CharArray`, numeric tensors and
  logical arrays; no new runtime types are required.
- The `strings/core` house pattern (`int2str`, `num2str`) covers descriptor,
  GPU/fusion spec, error-builder and registration conventions.
