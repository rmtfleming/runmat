# Feature Specification: Legacy Predicates — `isobject`, `isstr`, `isfolder`, `isdir`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `022-legacy-predicates`; export `SPECIFY_FEATURE=022-legacy-predicates`)

**Created**: 2026-07-22

**Status**: Batch 4 (Tier B batch 2) — gate approved; implementation complete

**Input**: Approved Tier B batch-2 exports `isobject` 0.2.0, `isstr` 0.2.0,
`isfolder` 0.2.0, `isdir` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Test whether a value is an object (Priority: P1)

RunMat users calling `isobject(A)` get a 1×1 logical telling whether `A` is an
object instance (a class instance that is not one of MATLAB's fundamental
types), matching the approved observed behaviour.

**Independent Test**: run each observed case from the export through the
builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** an object instance (an `inputParser`, feature 015), **When**
   `isobject` is called, **Then** the result is logical scalar `true`; numeric,
   char and cell inputs yield `false`.
   [isobject.logical-object-test, isobject.output-class]

### User Story 2 - Test whether a value is a character array (Priority: P1)

`isstr(A)` is the legacy equivalent of `ischar`: it returns a 1×1 logical that
is `true` only for a character array.

**Acceptance Scenarios**:

1. **Given** a character array, **When** `isstr` is called, **Then** the result
   is logical scalar `true`; numeric, a string scalar (`"..."`), and cell inputs
   yield `false`. [isstr.logical-char-test, isstr.output-class]

### User Story 3 - Test whether a path is an existing folder (Priority: P2)

`isfolder(path)` returns a 1×1 logical; `isdir(path)` is its legacy alias with
identical behaviour. An existing folder yields `true`; a nonexistent path
yields `false`.

**Acceptance Scenarios**:

1. **Given** an existing folder (`tempdir`), **When** `isfolder`/`isdir` is
   called, **Then** the result is logical scalar `true`.
   [isfolder.logical-folder-test, isfolder.output-class;
   isdir.logical-folder-test, isdir.output-class]
2. **Given** a path that does not exist, **When** `isfolder`/`isdir` is called,
   **Then** the result is logical scalar `false`.
   [isfolder.logical-folder-test; isdir.logical-folder-test]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-022-01** `isobject` MUST accept one argument (`tf = isobject(A)`)
  [isobject.signature-primary] and return class logical
  [isobject.output-class], 1×1, `true` for an object instance, observed `false`
  for numeric, char, and cell inputs [isobject.logical-object-test].
- **FR-022-02** `isstr` MUST accept one argument (`tf = isstr(A)`)
  [isstr.signature-primary] and return class logical [isstr.output-class], 1×1,
  `true` for a character array, observed `false` for numeric, a string scalar,
  and cell inputs [isstr.logical-char-test]. `isstr` is the legacy equivalent of
  `ischar` and MUST mirror its semantics exactly (a double-quoted string scalar
  is NOT char).
- **FR-022-03** `isfolder` MUST accept one path argument
  (`tf = isfolder(path)`) [isfolder.signature-primary] and return class logical
  [isfolder.output-class], 1×1; an existing folder MUST yield `true` and a path
  that does not exist MUST yield `false` [isfolder.logical-folder-test].
- **FR-022-04** `isdir` MUST accept one path argument (`tf = isdir(path)`)
  [isdir.signature-primary] and return class logical [isdir.output-class], 1×1,
  with behaviour identical to `isfolder` (existing folder → `true`, missing →
  `false`) [isdir.logical-folder-test]. `isdir` is the legacy equivalent of
  `isfolder`.
- **FR-022-05** Inputs not covered by the export (GPU values, string arrays,
  objects other than the observed kind, a file passed to `isfolder`/`isdir`,
  non-text inputs, …) MUST be handled by documented independent choice (see
  Unresolved behaviour) and MUST NOT be asserted as MATLAB-conformant.

### Key Entities

- Predicate result: `Value::Bool` (RunMat's 1×1 logical scalar).
- `isobject`/`isstr` inspect the runtime `Value` kind only (metadata-only, no
  filesystem or GPU execution).
- `isfolder`/`isdir` consult the filesystem through RunMat's existing runtime
  filesystem layer (`runmat_filesystem::metadata_async`, as `isfile` does),
  never through MATLAB.

## Success Criteria *(mandatory)*

- **SC-022-1**: Every observed case in the four exports reproduces exactly
  (value, class logical, size 1×1) in normative tests.
- **SC-022-2**: All four builtins registered and discoverable via the
  `#[runtime_builtin]` inventory machinery; `cargo test -p runmat-runtime`
  passes for their test modules.
- **SC-022-3**: Traceability: each FR cites ≥1 claim id; each normative test
  cites ≥1 FR/claim.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `isobject.q-fundamental`: confirmation of which built-in classes count as
  non-objects. Choice: predicate `true` for `Value::Object` and
  `Value::HandleObject` kinds; every fundamental kind (numeric/char/cell/
  struct/string/logical/function-handle/GPU) and every other object-adjacent
  kind (`Value::Listener`, `Value::MException`, `Value::ClassRef`) returns
  `false`, pending spec-side confirmation.
- `isstr.q-deprecated`: equivalence to `ischar` and any deprecation notice.
  Choice: `isstr` mirrors `ischar` exactly; no deprecation warning is emitted
  (unobserved).
- `isfolder.q-file`: behaviour when the path is a file rather than a folder.
  Choice: a file returns `false` (folder ≠ file), pending spec-side
  confirmation.
- `isdir.q-deprecated`: equivalence to `isfolder` and any deprecation notice.
  Choice: `isdir` delegates to the shared `isfolder` predicate; no deprecation
  warning is emitted (unobserved).
- String (`"..."`) and non-text inputs to `isfolder`/`isdir`: unobserved.
  Choice: a 1-element string is treated as path text (like existing RunMat fs
  builtins); non-text inputs return `false`.

## Assumptions

- RunMat's value model already distinguishes `Value::Object`,
  `Value::HandleObject`, `Value::CharArray`; no new runtime types are required.
- `isfolder`/`isdir` reuse the filesystem provider abstraction used by
  `io/repl_fs/isfile.rs` so WASM/native behave consistently.
- RunMat's `inputParser` (feature 015) is represented as a
  `Value::HandleObject`, satisfying the observed `isobject(inputParser)` case at
  the value-kind level.
