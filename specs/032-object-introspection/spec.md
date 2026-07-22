# Feature Specification: Object Introspection — `properties`, `methods`, `what`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`032-object-introspection`; export `SPECIFY_FEATURE=032-object-introspection`)

**Created**: 2026-07-22

**Status**: Batch 5 (Tier B batch 3) — combined review pending

**Input**: Approved Tier B batch-3 exports `properties` 0.2.0, `methods`
0.2.0, `what` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - List an object's properties from MATLAB code (Priority: P1)

RunMat users calling `properties(A)` on an object (or class) get an N×1 cell
array of char vectors naming the object's public properties, matching the
approved observed *shape and element type*.

**Independent Test**: construct RunMat's own `inputParser` (feature 015) and
call `properties` on it; confirm the result is a column cell whose elements
are all char vectors.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** an object, **When** `properties(A)` is called, **Then** the
   result is an N×1 cell array whose elements are char vectors (iscellstr
   true). [properties.signature-primary, properties.cellstr-names]

### User Story 2 - List an object's methods (Priority: P1)

`methods(A)` returns an N×1 cell array of char vectors naming the object's
public methods.

**Acceptance Scenarios**:

1. **Given** an object, **When** `methods(A)` is called, **Then** the result
   is an N×1 cell array whose elements are char vectors (iscellstr true).
   [methods.signature-primary, methods.cellstr-names]

### User Story 3 - Describe the MATLAB files in a folder (Priority: P2)

`what(folder)` returns a 1×1 struct describing the MATLAB-related files in a
folder.

**Acceptance Scenarios**:

1. **Given** a folder path, **When** `what(folder)` is called, **Then** the
   result has class `struct`, size 1×1, and exposes the documented fields.
   [what.signature-primary, what.output-class, what.folder-struct]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-032-01** `properties` MUST accept the primary form
  `c = properties(A)` [properties.signature-primary] and return an N×1 cell
  array whose elements are char vectors [properties.cellstr-names]. The cell
  holds the property names of the *actual RunMat object/class*. The specific
  member list and count are object-specific and MUST NOT be asserted (see
  **Value non-assertion**).
- **FR-032-02** `methods` MUST accept the primary form `c = methods(A)`
  [methods.signature-primary] and return an N×1 cell array whose elements are
  char vectors [methods.cellstr-names]. The cell holds the method names of the
  actual RunMat object/class. The specific member list and count are
  object-specific and MUST NOT be asserted.
- **FR-032-03** `what` MUST accept the primary form `s = what(folder)`
  [what.signature-primary] and return class `struct` [what.output-class],
  size 1×1, describing the folder's MATLAB-related files [what.folder-struct].
  The struct MUST expose a documented field set; the specific field names and
  listed contents are folder-/environment-dependent and MUST NOT be asserted
  as MATLAB-conformant beyond the presence of the documented fields.
- **FR-032-04** Inputs and forms not covered by the exports (class-name text
  for `properties`/`methods`, the no-argument form of `what`, non-object /
  non-text inputs, empty member sets) MUST be handled by documented
  independent choice (see **Unresolved behaviour**) and MUST NOT be asserted
  as MATLAB-conformant.

### Key Entities

- Property / method name list: RunMat `Value::Cell` (N×1) of
  `Value::CharArray` row vectors, sourced from the class registry
  (`ClassDef`) via `runmat_builtins::get_class` and the inheritance chain.
- `what` result: RunMat `Value::Struct` (1×1) whose file-list fields are N×1
  cellstr columns and whose `path` field is a char row; populated by scanning
  the folder through RunMat's runtime filesystem layer (`runmat_filesystem`),
  never through MATLAB.

## Value non-assertion *(mandatory framing — gate-ratified)*

The approved exports observed concrete member counts and a concrete result
struct, but those are properties of MATLAB's own objects/environment, not of
the interface contract this feature implements:

- `properties(inputParser)` was observed as a **9×1** cell and
  `methods(inputParser)` as a **19×1** cell. These counts, and the specific
  member *names*, are MATLAB-object-specific. RunMat asserts only the
  **class** (`cell`), the **column shape** (N×1), and that **every element is
  a char vector**. RunMat's `inputParser` (feature 015) legitimately reports a
  different member set (e.g. `Results`, `UsingDefaults`; `addRequired`,
  `addParameter`, `parse`), and the tests reproduce *that* actual set, never
  MATLAB's count.
- `what(tempdir)` was observed as a **1×1 struct**; the export explicitly
  leaves the field names (`what.q-fields`) and listed contents
  (`what.q-content`) unresolved. RunMat asserts the struct **class**, the
  **1×1 size**, and the **presence of its documented fields**; it does not
  assert that MATLAB has exactly this field set nor that any particular file
  appears.

This mirrors the value-/permutation-non-assertion framing used by the sparse
ordering features (`colamd`, `dissect`), where only the contract-level
invariants — not environment-specific concrete values — are asserted.

## Success Criteria *(mandatory)*

- **SC-032-1**: Each export's observed *shape/class* invariant reproduces:
  `properties`/`methods` return a column `cell` of char vectors; `what`
  returns a 1×1 `struct` with the documented fields.
- **SC-032-2**: All three builtins are registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime` passes for their
  test modules.
- **SC-032-3**: Traceability: each FR cites ≥1 claim id; each normative test
  cites ≥1 FR; the value non-assertion is enforced by explicit
  count-non-assertion tests.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `properties.q-class-dependent` / `methods.q-class-dependent`: the specific
  names depend on the class. Choice: return the class registry's public
  (`get_access`/`access == Public`) declared members, de-duplicated across the
  inheritance chain, in ascending order. Declaration order and the
  public-only filter are documented choices, not asserted as MATLAB order.
- `methods.q-full`: the `'-full'` option is unobserved and is **not**
  implemented (unsupported form; not accepted).
- `what.q-fields`: field names unresolved. Choice: documented field set
  `m`, `mat`, `mex`, `mlx`, `mlapp`, `classes`, `packages`, `path`.
- `what.q-content`: listed files depend on the folder. Choice: classify by
  extension (`.m`→m, `.mat`→mat, `.mex*`→mex, `.mlx`→mlx, `.mlapp`→mlapp) and
  by folder prefix (`@Name`→classes, `+name`→packages); other entries are
  ignored. Contents are never asserted for uncontrolled folders.
- Class-name text form for `properties`/`methods` (unobserved): accepted and
  resolved through the class registry; an unregistered name errors with a
  RunMat identifier (wording is RunMat's own, unobserved).
- No-argument `what` (unobserved): scans the current directory.
- Non-object / non-text inputs: rejected with a RunMat identifier.

## Assumptions

- RunMat's class registry (`ClassDef` with `PropertyDef`/`MethodDef`) already
  models the members `properties`/`methods` must report; no new runtime type
  is required. The `inputParser` class (feature 015) provides a concrete
  registered class for the observed-shape tests.
- `what` reuses the filesystem abstraction used by `io/repl_fs/dir.rs`
  (`runmat_filesystem`) so native/WASM behave consistently and no MATLAB is
  invoked.
