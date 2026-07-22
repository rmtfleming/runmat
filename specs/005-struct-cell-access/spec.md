# Feature Specification: Struct & Cell Access — `fields`, `num2cell`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `005-struct-cell-access`; export `SPECIFY_FEATURE=005-struct-cell-access`)

**Created**: 2026-07-22

**Status**: Batch 1 — combined review pending

**Input**: Approved Tier A exports `fields` 0.2.0, `num2cell` 0.2.0 (see
`provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - List struct field names (Priority: P1)

RunMat users calling `fields(s)` on a struct get an N×1 cell array of
field-name character vectors (0×1 when the struct has no fields), matching
the approved observed behaviour. The approved summary describes `fields` as a
legacy equivalent of `fieldnames`.

**Independent Test**: run each observed case from the export through the
builtin and compare class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** `struct('a',1,'b',2)`, **When** `fields` is called, **Then** the
   result is a 2×1 cell of field-name char vectors. [fields.output-class,
   fields.cell-fieldnames]
2. **Given** `struct()` (no fields), **When** `fields` is called, **Then**
   the result is a 0×1 cell. [fields.output-class, fields.cell-fieldnames]

### User Story 2 - Convert an array to a cell array of its elements (Priority: P1)

`num2cell(A)` returns a cell array with one cell per element of `A`,
preserving the input shape; `num2cell(A, dim)` groups elements along the
given dimension.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** `[10 20 30]`, **When** `num2cell` is called, **Then** the
   result is a 1×3 cell, one cell per element. [num2cell.output-class,
   num2cell.cell-grouping]
2. **Given** `[1 2; 3 4]`, **When** `num2cell` is called, **Then** the
   result is a 2×2 cell preserving the input shape. [num2cell.cell-grouping]
3. **Given** `[1 2; 3 4]` and `dim = 1`, **When** `num2cell` is called,
   **Then** the result is a 1×2 cell. [num2cell.cell-grouping]
4. **Given** scalar `7`, **When** `num2cell` is called, **Then** the result
   is a 1×1 cell. [num2cell.cell-grouping]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-005-01** `fields` MUST accept one struct argument (`c = fields(s)`)
  [fields.signature-primary] and return class cell [fields.output-class],
  shaped N×1 with one field-name char vector per field, 0×1 for a struct
  with no fields [fields.cell-fieldnames].
- **FR-005-02** (summary-derived) `fields` behaves as a legacy equivalent of
  `fieldnames` per the approved summary; RunMat implements it by delegating
  to its `fieldnames` builtin. Exact equivalence/deprecation status is
  unresolved on the specification side (`fields.q-alias`).
- **FR-005-03** `fields` inputs not covered by the export (non-struct
  inputs, struct arrays, objects, name ordering within the cell) MUST be
  handled by documented independent choice (see Unresolved behaviour) and
  MUST NOT be asserted as MATLAB-conformant.
- **FR-005-04** `num2cell` MUST accept one array argument
  (`C = num2cell(A)`) [num2cell.signature-primary] and return class cell
  [num2cell.output-class] with, by default, one cell per element preserving
  the input shape: observed 1×3 for `[10 20 30]`, 2×2 for `[1 2; 3 4]`, 1×1
  for scalar `7` [num2cell.cell-grouping].
- **FR-005-05** `num2cell` MUST accept an optional dimension argument
  (`C = num2cell(A, dim)`); `dim = 1` on a 2×2 matrix MUST produce a 1×2
  cell [num2cell.cell-grouping]. The grouped cell contents (each a 2×1
  column) follow from the approved claim statement ("groups elements along
  that dimension") and are tagged summary-derived — the observation records
  only the outer class/size.
- **FR-005-06** `num2cell` inputs and forms not covered by the export (empty
  arrays, dimension vectors, char/logical/complex/string inputs, error
  conditions, GPU values) MUST be handled by documented independent choice
  (see Unresolved behaviour) and MUST NOT be asserted as MATLAB-conformant.

### Key Entities

- `fields` result: `Value::Cell` (N×1) of `Value::CharArray` rows, produced
  by the existing `fieldnames` builtin via runtime dispatch
  (`call_builtin_async`), the same delegation pattern `bounds` uses for
  `min`/`max`.
- `num2cell` result: `Value::Cell` whose elements are RunMat scalar values
  (`Value::Num`, `Value::Bool`, `Value::Complex`, `Value::String`, 1×1
  `CharArray`) or tensor slices for grouped dimensions. RunMat tensors are
  column-major; cell-array storage is row-major — the conversion iterates
  accordingly.

## Success Criteria *(mandatory)*

- **SC-005-1**: Every observed case in the two exports reproduces exactly
  (class cell, observed size) in normative tests.
- **SC-005-2**: Both builtins registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime` passes for
  their test modules.
- **SC-005-3**: Traceability: each FR cites ≥1 claim id (or is tagged
  summary-derived/unresolved); each normative test cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `fields.q-alias`: exact equivalence with `fieldnames` and deprecation
  status unconfirmed. Choice: full delegation to RunMat's `fieldnames`
  (sorted N×1 names; struct arrays yield the sorted union; objects/handles
  supported as `fieldnames` supports them; non-struct inputs error with the
  delegate's message).
- `fields` name ordering: the observation records only the 2×1 size, not the
  cell contents. Choice: sorted order (inherited from RunMat `fieldnames`).
- `num2cell.q-shape`: cell-array shape for each form, especially with dim,
  unconfirmed beyond the four observed cases. Choices: empty input preserves
  the empty shape (`[]` → 0×0 cell); a dimension vector groups every listed
  dimension (`[1 2]` on 2×2 → 1×1 cell holding the whole matrix); dimensions
  beyond `ndims` are treated as trailing singletons; char inputs yield char
  cells sliced row-major; non-positive/fractional dims and >2 arguments
  error; GPU inputs are gathered to the host first.

## Assumptions

- RunMat value model already distinguishes `Value::Struct`, `Value::Cell`,
  and the numeric/logical/char/string array kinds; no new runtime types
  required.
- `fields` delegates through the existing builtin registry
  (`call_builtin_async`), exercising the same dispatch machinery as user
  code.
- `num2cell` mirrors the input-kind handling of the existing `mat2cell`
  builtin (numeric, complex, logical, string, char; GPU values gathered).
