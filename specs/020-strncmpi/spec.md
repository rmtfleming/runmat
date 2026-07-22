# Feature Specification: Case-Insensitive Prefix Comparison — `strncmpi`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`020-strncmpi`; export `SPECIFY_FEATURE=020-strncmpi`)

**Created**: 2026-07-22

**Status**: Tier B batch 1 — gate approved; implementation authorized

**Input**: Approved Tier B export `strncmpi` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Compare leading characters of two texts ignoring case (Priority: P1)

RunMat users calling `strncmpi(s1, s2, n)` get a logical result that is true
when the first `n` characters of the two texts match ignoring case, and
false when a character within the first `n` differs — matching the approved
observed behaviour exactly.

**Independent Test**: run each observed case from the export through the
builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** `'Hello'` and `'help'` with `n = 3`, **When** `strncmpi` is
   called, **Then** the result is a 1×1 logical `true` (first three
   characters match ignoring case). [strncmpi.signature-primary,
   strncmpi.output-class, strncmpi.case-insensitive-prefix]
2. **Given** `'abc'` and `'abd'` with `n = 2`, **When** `strncmpi` is
   called, **Then** the result is a 1×1 logical `true` (equal two-character
   prefix). [strncmpi.output-class, strncmpi.case-insensitive-prefix]
3. **Given** `'abc'` and `'xyz'` with `n = 1`, **When** `strncmpi` is
   called, **Then** the result is a 1×1 logical `false` (different first
   character). [strncmpi.output-class, strncmpi.case-insensitive-prefix]

### User Story 2 - Filter text collections by case-insensitive prefix (Priority: P2)

Users pass cell arrays of character vectors or string arrays and receive an
element-wise logical array, following RunMat's existing `strncmp` container
and broadcasting semantics with case folding. *(Unobserved in the export —
documented independent choice; see Unresolved behaviour.)*

## Requirements *(mandatory)*

### Functional Requirements

- **FR-020-01** `strncmpi` MUST accept the three-argument form
  `tf = strncmpi(s1, s2, n)` [strncmpi.signature-primary].
- **FR-020-02** The output MUST have class logical
  [strncmpi.output-class]; for scalar text inputs the result is 1×1
  (RunMat's `Value::Bool`).
- **FR-020-03** For the observed scalar text inputs, the result MUST be
  true when the first `n` characters match ignoring case and false when a
  character within the first `n` differs
  [strncmpi.case-insensitive-prefix]. All three observed cases MUST
  reproduce exactly (value, class, size).
- **FR-020-04** Inputs and edge cases not covered by the export (cell
  arrays, string arrays, char-array rows, missing string values, `n = 0`,
  strings shorter than `n`, invalid `n` values, non-text inputs, GPU
  operands) MUST be handled by documented independent choice mirroring
  RunMat's existing `strncmp` semantics with the case folding used by
  RunMat's `strcmpi`, and MUST NOT be asserted as MATLAB-conformant.
  [strncmpi.q-arrays]

### Key Entities

- Comparison result: `Value::Bool` for scalar comparisons, otherwise a
  `Value::LogicalArray` shaped by MATLAB-style broadcasting (independent
  choice, same machinery as `strncmp`/`strcmpi`).
- Text containers: char rows, string scalars, string arrays, cell arrays of
  character vectors — decoded through the existing shared
  `TextCollection` utilities.

## Success Criteria *(mandatory)*

- **SC-020-1**: Every observed case in the `strncmpi` 0.2.0 export
  reproduces exactly (value, class logical, size 1×1) in normative tests.
- **SC-020-2**: `strncmpi` is registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime` passes for its
  test module.
- **SC-020-3**: Traceability: each FR cites ≥1 claim id; each normative
  test cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `strncmpi.q-arrays` (carried from the export): behaviour with cell-array
  inputs (element-wise). Choice: element-wise logical array with MATLAB
  broadcast semantics, exactly mirroring RunMat's `strncmp`, with each
  element comparison case-folded.
- Length handling beyond the observed cases (unobserved). Choice: copy
  RunMat's `strncmp` rule — `n = 0` is always true; if either string ends
  before `n` characters while the other continues, the result is false; if
  both strings end simultaneously before `n` characters, the result is
  true.
- Missing string values (unobserved). Choice: as in `strncmp` — a missing
  element compares false when `n > 0` and true when `n = 0`.
- Invalid `n` (negative, non-integer, non-finite, non-scalar, non-numeric)
  errors with `RunMat:strncmpi:InvalidPrefixLength`; non-text inputs error
  with `RunMat:strncmpi:InvalidInput`; non-broadcastable shapes error with
  `RunMat:strncmpi:ShapeMismatch` (all mirroring `strncmp`).
- Case folding for non-ASCII characters (unobserved). Choice: Unicode
  lowercase folding per character, consistent with RunMat's `strcmpi`
  (which folds whole texts with the same lowercase mapping).

## Assumptions

- The shared text utilities (`TextCollection`, `logical_result`) and
  broadcast helpers used by `strcmp`/`strcmpi`/`strncmp` are reused; no new
  runtime types are required.
- GPU operands are gathered to the host before comparison (`accel = sink`),
  as in `strncmp`.
