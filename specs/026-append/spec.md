# Feature Specification: `append` — string/char concatenation

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`026-append`; export `SPECIFY_FEATURE=026-append`)

**Created**: 2026-07-22

**Status**: Batch 4 (Tier B batch 2) — combined review pending

**Input**: Approved Tier B export `append` 0.2.0 (see `provenance.md`).

## Overview

`append` (introduced R2020b) concatenates text pieces in order and returns the
combined text. It is the string-concatenation function — despite the name it is
unrelated to tables. It differs from `strcat` in one observable way: `append`
preserves trailing whitespace on character-array inputs, whereas `strcat` trims
it.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Concatenate text from MATLAB code (Priority: P1)

RunMat users calling `append(str1, str2, ...)` get the arguments joined in
order, with the container class following MATLAB's rule: any string input makes
the result a string, all-char inputs make the result a char row.

**Independent Test**: run each observed case from the export through the builtin
and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** two string scalars `"a"`, `"b"`, **When** `append` is called,
   **Then** the result is the string scalar `"ab"` (class `string`, size
   `[1 1]`). [append.signature-primary, append.concatenates-text]
2. **Given** three string scalars `"x"`, `"y"`, `"z"`, **When** `append` is
   called, **Then** the result is the string scalar `"xyz"` (class `string`,
   size `[1 1]`). [append.concatenates-text]
3. **Given** two char rows `'a'`, `'b'`, **When** `append` is called, **Then**
   the result is the char row `'ab'` (class `char`, size `[1 2]`).
   [append.concatenates-text]

### User Story 2 - Preserve trailing whitespace (Priority: P2)

Unlike `strcat`, `append` does not strip trailing spaces from char inputs.

**Acceptance Scenario** *(unobserved; documented independent choice)*:

1. **Given** `'hi '` and `'there'`, **When** `append` is called, **Then** the
   result is `'hi there'` (the trailing space is preserved). *(Tagged
   `unresolved_choice`; distinguishes `append` from `strcat`.)*

## Requirements *(mandatory)*

### Functional Requirements

- **FR-026-01** `append` MUST accept the variadic form
  `str = append(str1, str2, ...)` [append.signature-primary] and concatenate
  the text arguments left-to-right [append.concatenates-text].
- **FR-026-02** `append` MUST return class `string` (string scalar for scalar
  inputs) when **any** argument is a string, and class `char` (a char row) when
  **all** arguments are character arrays [append.concatenates-text].
- **FR-026-03** For the observed cases, the result value, class and size MUST
  reproduce exactly: `append("a","b") → "ab"` `[1 1]` string;
  `append("x","y","z") → "xyz"` `[1 1]` string; `append('a','b') → 'ab'`
  `[1 2]` char [append.concatenates-text].
- **FR-026-04** (documented independent choice) `append` MUST preserve trailing
  whitespace on char inputs (no trimming), in contrast to `strcat`. Unobserved
  for `append`; recorded under Unresolved behaviour.
- **FR-026-05** (documented independent choice) Inputs and shapes not covered by
  the export — string arrays (element-wise concatenation with implicit
  expansion), a single argument, empty text arrays, non-text inputs, and
  broadcast-incompatible shapes — MUST be handled by documented independent
  choice (see Unresolved behaviour) and MUST NOT be asserted as
  MATLAB-conformant.

### Key Entities

- Result value: `Value::String` (scalar), `Value::StringArray` (non-scalar
  string result), or `Value::CharArray` (all-char result) in RunMat's value
  model.
- Concatenation and implicit expansion reuse RunMat's shared broadcast helpers
  (`builtins/common/broadcast`), the same machinery `strcat` uses.

## Success Criteria *(mandatory)*

- **SC-026-1**: Every observed case in the export reproduces exactly (value,
  class, size) in normative tests.
- **SC-026-2**: `append` is registered and discoverable through the
  `#[runtime_builtin]` inventory (same mechanism as all existing builtins);
  `cargo test -p runmat-runtime` passes for its test module.
- **SC-026-3**: Traceability: each FR cites ≥1 claim id or is explicitly tagged
  as a documented independent choice; each normative test cites ≥1 FR/claim.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `append.q-release`: the exact introducing release is carried from the export
  as unresolved; RunMat implements `append` unconditionally (no release gate).
- `append.q-types`: output type across the full char/string/mixed input matrix
  is only partially observed. Choice implemented: any string → string result;
  all char → char row; this is asserted only for the observed subset (FR-026-03)
  and applied by construction elsewhere.
- Trailing whitespace on char inputs is **preserved** (FR-026-04) — the defining
  contrast with `strcat`; unobserved for `append`.
- String arrays concatenate **element-wise** with MATLAB implicit expansion
  (scalar broadcasting); a non-scalar string result is a `string` array whose
  shape is the broadcast shape.
- A **single** argument returns that argument's text unchanged (the variadic
  tail may be empty): `append("solo") → "solo"`.
- **Empty** text arrays propagate their (possibly zero) shape through the
  broadcast, yielding an empty result of the resolved class.
- **Non-text** inputs (numeric, logical, cell, struct, GPU tensors, …) are
  rejected with `RunMat:append:InvalidInput`. `append` is scoped to the
  observed string/char classes; cellstr support is intentionally out of scope
  (Minimal Scope) and MUST return to the specification side if required.
- **Broadcast-incompatible** shapes are rejected with
  `RunMat:append:SizeMismatch`.
- **Zero** arguments are rejected with `RunMat:append:NotEnoughInputs`.

## Assumptions

- RunMat's value model already distinguishes `Value::String`,
  `Value::StringArray` and `Value::CharArray`; no new runtime types required.
- Char arrays are modelled one text piece per row for concatenation, consistent
  with RunMat's existing text-concatenation builtins (`strcat`).
