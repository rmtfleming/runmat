# Feature Specification: Shape & Storage Predicates — `isrow`, `iscolumn`, `issparse`, `iscellstr`, `istable`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`016-shape-storage-predicates`; export
`SPECIFY_FEATURE=016-shape-storage-predicates`)

**Created**: 2026-07-22

**Status**: Tier B batch 1 — gates approved (human-loop record 2026-07-22)

**Input**: Approved Tier B batch 1 exports `isrow` 0.2.0, `iscolumn` 0.2.0,
`issparse` 0.2.0, `iscellstr` 0.2.0, `istable` 0.2.0 (see `provenance.md`;
source commit `e46587ae1b78237e0f43cd55f678de01a28495a4`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Test a value's shape from MATLAB code (Priority: P1)

RunMat users calling `isrow(A)` / `iscolumn(A)` get a 1×1 logical telling
whether `A` is a row vector (1-by-N) / column vector (N-by-1), matching the
approved observed behaviour. A scalar counts as both a row and a column.

**Independent Test**: run each observed case from the export through the
builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** `[1 2 3]`, **When** `isrow` is called, **Then** the result is
   logical scalar `true`; `[1;2;3]` and `[1 2;3 4]` yield `false`; `5`
   yields `true`. [isrow.logical-row-test, isrow.output-class]
2. **Given** `[1;2;3]`, **When** `iscolumn` is called, **Then** the result is
   logical scalar `true`; `[1 2 3]` and `[1 2;3 4]` yield `false`; `5`
   yields `true`. [iscolumn.logical-column-test, iscolumn.output-class]

### User Story 2 - Test a value's storage class (Priority: P1)

`issparse(A)` returns a 1×1 logical that is `true` exactly when `A` uses
sparse storage (including an empty sparse array) and `false` for full
(dense) inputs.

**Acceptance Scenarios**:

1. **Given** `sparse([1 0; 0 2])`, **When** `issparse` is called, **Then**
   the result is logical scalar `true`; `[1 2; 3 4]` yields `false`;
   `sparse([])` yields `true`. [issparse.logical-sparse-test,
   issparse.output-class]

### User Story 3 - Test for a cell array of character vectors (Priority: P2)

`iscellstr(A)` returns a 1×1 logical that is `true` when `A` is a cell array
whose every element is a character vector (the empty cell `{}` included) and
`false` when any element is non-char or the input is a plain char array.

**Acceptance Scenarios**:

1. **Given** `{'a','bc'}`, **When** `iscellstr` is called, **Then** the
   result is logical scalar `true`; `{'a',1}` and `'abc'` yield `false`;
   `{}` yields `true`. [iscellstr.logical-cellstr-test,
   iscellstr.output-class]

### User Story 4 - Test for a table (Priority: P3)

`istable(A)` returns a 1×1 logical that is `true` for a table and `false`
for non-table inputs (numeric, cell observed).

**Acceptance Scenarios**:

1. **Given** `[1 2; 3 4]` or `{1, 2}`, **When** `istable` is called,
   **Then** the result is logical scalar `false`.
   [istable.logical-table-test, istable.output-class]
2. **Given** a table, **When** `istable` is called, **Then** the result is
   `true` [istable.logical-table-test, case `table`]. **Documented note —
   unreachable in RunMat today**: RunMat's runtime value model has no table
   type (Blocker B-010-1 in `specs/010-table-conversion/spec.md`), so no
   table value can be constructed and every constructible input correctly
   reports `false`. The gate-ratified decision is an always-false
   implementation; this observed true-case becomes implementable only when
   a table type lands, at which point this feature returns to planning.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-016-01** `isrow` MUST accept one argument (`tf = isrow(A)`)
  [isrow.signature-primary] and return class logical [isrow.output-class],
  1×1, `true` exactly for 2-D values whose first dimension is 1 (a scalar
  counts as a row), observed `false` for a column vector and a 2-D matrix
  [isrow.logical-row-test].
- **FR-016-02** `iscolumn` MUST accept one argument (`tf = iscolumn(A)`)
  [iscolumn.signature-primary] and return class logical
  [iscolumn.output-class], 1×1, `true` exactly for 2-D values whose second
  dimension is 1 (a scalar counts as a column), observed `false` for a row
  vector and a 2-D matrix [iscolumn.logical-column-test].
- **FR-016-03** `issparse` MUST accept one argument (`tf = issparse(A)`)
  [issparse.signature-primary] and return class logical
  [issparse.output-class], 1×1, `true` exactly for sparse-storage inputs
  including an empty sparse array, observed `false` for full inputs
  [issparse.logical-sparse-test].
- **FR-016-04** `iscellstr` MUST accept one argument (`tf = iscellstr(A)`)
  [iscellstr.signature-primary] and return class logical
  [iscellstr.output-class], 1×1, `true` when every cell element is a
  character vector (including the empty cell `{}`), observed `false` when
  any element is non-char or the input is a plain char array
  [iscellstr.logical-cellstr-test].
- **FR-016-05** `istable` MUST accept one argument (`tf = istable(A)`)
  [istable.signature-primary] and return class logical
  [istable.output-class], 1×1, observed `false` for numeric and cell inputs
  [istable.logical-table-test]. Because RunMat has no table type, the
  implementation returns `false` for every value kind — correct for every
  constructible input; the observed true-case (case `table`) is recorded as
  unreachable (see User Story 4, documented note).
- **FR-016-06** Inputs not covered by the exports (GPU values, N-D arrays,
  empty shapes for `isrow`/`iscolumn`, string elements for `iscellstr`,
  char matrices inside cells, …) MUST be handled by documented independent
  choice (see Unresolved behaviour) and MUST NOT be asserted as
  MATLAB-conformant.

### Key Entities

- Predicate result: `Value::Bool` (RunMat's logical scalar ≡ 1×1 logical).
- Shape source for `isrow`/`iscolumn`: RunMat's MATLAB-visible dimension
  metadata (`builtins::common::shape::value_dimensions`), which answers GPU
  handles from shape metadata without gathering whenever the provider
  supplies it.
- Storage source for `issparse`: the runtime value kind
  (`Value::SparseTensor` is RunMat's sole sparse representation).

## Success Criteria *(mandatory)*

- **SC-016-1**: Every observed case in the five exports that is
  constructible in RunMat (17 of 18; the `istable` case `table` is
  unreachable — documented note) reproduces exactly (value, class logical,
  size 1×1) in normative tests.
- **SC-016-2**: All five builtins registered and discoverable via the
  builtin registry; `cargo test -p runmat-runtime` passes for their test
  modules.
- **SC-016-3**: Traceability: each FR cites ≥1 claim id; each normative
  test cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `isrow.q-empty` / `iscolumn.q-empty`: empty-shaped inputs unobserved.
  Choice: the size rule is applied uniformly — `1×0` is a row (not a
  column), `0×1` is a column (not a row), `0×0` is neither.
- N-D inputs (ndims > 2) for `isrow`/`iscolumn`: unobserved. Choice:
  `false` (the observed rule is stated for 2-D shapes; trailing singleton
  dimensions are not collapsed, consistent with `isvector`).
- GPU tensors: unobserved. Choice: `isrow`/`iscolumn` answer from handle
  shape metadata (gathering only if a provider omits shape); `issparse`,
  `iscellstr`, `istable` answer `false` from the value kind without any
  gather (a GPU tensor is dense numeric storage, not a cell or table).
- `issparse.q-output` (confirm output class/size over wider inputs):
  observed cases already pin logical 1×1; carried unresolved on the
  specification side. Choice: every non-sparse runtime kind returns `false`.
- `iscellstr.q-strings`: cell of string (not char) elements unobserved.
  Choice: `Value::String` elements (RunMat's internal string-scalar text
  representation) count as text and report `true`; a `Value::StringArray`
  input is not a cell and reports `false`.
- Char matrices inside cells (element is 2-D char, not a vector):
  unobserved. Choice: the element test is kind-level (char-ness, mirroring
  RunMat's `ischar`), so a char-matrix element reports `true`.
- `istable.q-timetable`: timetable inputs unobserved and not constructible
  in RunMat; nothing to implement.

## Assumptions

- RunMat value model already distinguishes `Value::SparseTensor`,
  `Value::Cell`, `Value::CharArray`; no new runtime types required.
- `Value::Bool` is RunMat's 1×1 logical, as established by features 002/006.
- RunMat has no table value kind (verified: `builtins/table/` is an empty
  stub; Blocker B-010-1) — basis for the ratified always-false `istable`.
