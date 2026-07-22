# Feature Specification: Type Predicates — `iscell`, `isstruct`, `isfile`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `002-type-predicates`; export `SPECIFY_FEATURE=002-type-predicates`)

**Created**: 2026-07-22

**Status**: Batch 1 — combined review pending

**Input**: Approved Tier A exports `iscell` 0.2.0, `isstruct` 0.2.0,
`isfile` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Test a value's kind from MATLAB code (Priority: P1)

RunMat users calling `iscell(A)` / `isstruct(A)` get a 1×1 logical telling
whether `A` is a cell array / struct (array), matching the approved observed
behaviour.

**Independent Test**: run each observed case from the export through the
builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** a cell array (including empty `{}`), **When** `iscell` is
   called, **Then** the result is logical scalar `true`; numeric, char and
   struct inputs yield `false`. [iscell.logical-cell-test, iscell.output-class]
2. **Given** a scalar struct or struct array, **When** `isstruct` is called,
   **Then** the result is logical scalar `true`; numeric and cell inputs
   yield `false`. [isstruct.logical-struct-test, isstruct.output-class]

### User Story 2 - Test whether a path is an existing file (Priority: P2)

`isfile(path)` returns a 1×1 logical; a nonexistent path yields `false`.

**Acceptance Scenarios**:

1. **Given** a path that does not exist, **When** `isfile` is called,
   **Then** the result is logical scalar `false`. [isfile.missing-false,
   isfile.output-class]
2. **Given** an existing regular file, **When** `isfile` is called, **Then**
   the result is `true`. *(Summary-derived: approved spec summary text; no
   backing observation case — see Unresolved. Test tagged `summary_derived`.)*

## Requirements *(mandatory)*

### Functional Requirements

- **FR-002-01** `iscell` MUST accept one argument (`tf = iscell(A)`)
  [iscell.signature-primary] and return class logical
  [iscell.output-class], 1×1, `true` exactly for cell arrays (including
  `{}`), observed `false` for numeric, char, struct
  [iscell.logical-cell-test].
- **FR-002-02** `isstruct` MUST accept one argument [isstruct.signature-primary]
  and return class logical [isstruct.output-class], 1×1, `true` for scalar
  structs and struct arrays, observed `false` for numeric and cell inputs
  [isstruct.logical-struct-test].
- **FR-002-03** `isfile` MUST accept one path argument [isfile.signature-primary]
  and return class logical [isfile.output-class], 1×1; a path that does not
  exist MUST yield `false` [isfile.missing-false].
- **FR-002-04** (summary-derived) `isfile` SHOULD return `true` for an
  existing regular file, per the approved summary; no observation case backs
  this yet.
- **FR-002-05** Inputs not covered by the export (GPU values, string arrays,
  objects, folders for `isfile`, …) MUST be handled by documented independent
  choice (see Unresolved behaviour) and MUST NOT be asserted as
  MATLAB-conformant.

### Key Entities

- Predicate result: `Value::Bool` (RunMat's logical scalar).
- `isfile` consults the filesystem through RunMat's existing runtime
  filesystem layer (as `exist` does), never through MATLAB.

## Success Criteria *(mandatory)*

- **SC-002-1**: Every observed case in the three exports reproduces exactly
  (value, class logical, size 1×1) in normative tests.
- **SC-002-2**: All three builtins registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime` passes for their
  test modules.
- **SC-002-3**: Traceability: each FR cites ≥1 claim id; each normative test
  cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `iscell.q-output` / `isstruct.q-output`: confirmation over wider input set.
  Choice: predicate true only for `Value::Cell` / `Value::Struct` kinds;
  every other runtime kind returns false.
- `isfile.q-existing`: existing-file → true unobserved (FR-002-04 tagged).
- `isfile.q-folder`: folder behaviour unobserved. Choice: folders return
  `false` (file ≠ folder), pending spec-side confirmation.
- String (`"..."`) inputs to `isfile`: unobserved. Choice: accepted and
  treated as path text, like existing RunMat fs builtins.

## Assumptions

- RunMat value model already distinguishes `Value::Cell`, `Value::Struct`;
  no new runtime types required.
- `isfile` reuses the filesystem provider abstraction used by
  `io/repl_fs/exist.rs` so WASM/native behave consistently.
