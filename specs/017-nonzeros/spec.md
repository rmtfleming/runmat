# Feature Specification: `nonzeros`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`017-nonzeros`; export `SPECIFY_FEATURE=017-nonzeros`)

**Created**: 2026-07-22

**Status**: Gate-approved pipeline (Tier B batch 1); implemented

**Input**: Approved Tier B export `nonzeros` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Collect the nonzero elements of an array (Priority: P1)

RunMat users calling `v = nonzeros(A)` on a full or sparse numeric array get
a double column vector containing exactly the nonzero elements of `A`, in
column-major order, matching the approved observed behaviour.

**Independent Test**: run each observed case from the export through the
builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** the row vector `[0 1 0 2]`, **When** `nonzeros` is called,
   **Then** the result is `[1;2]` (2×1 double).
   [nonzeros.signature-primary, nonzeros.output-class,
   nonzeros.column-major-order]
2. **Given** the matrix `[1 2; 0 4]`, **When** `nonzeros` is called, **Then**
   the result is `[1;2;4]` (3×1 double). [nonzeros.output-class,
   nonzeros.column-major-order]
3. **Given** the matrix `[1 2; 3 0]`, **When** `nonzeros` is called, **Then**
   the result is `[1;3;2]` (3×1 double) — column-major traversal `1,3,2,0`;
   a row-major traversal would give `[1;2;3]`, so this case discriminates
   the two orderings. [nonzeros.column-major-order]
4. **Given** `sparse([0 3 0])`, **When** `nonzeros` is called, **Then** the
   result is `3` (1×1 double, not sparse). [nonzeros.output-class,
   nonzeros.column-major-order]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-017-01** `nonzeros` MUST accept exactly one argument
  (`v = nonzeros(A)`) [nonzeros.signature-primary].
- **FR-017-02** The output MUST have class double and be a column vector
  (n×1; a single nonzero yields 1×1, which RunMat represents as
  `Value::Num`); a sparse input yields a full (non-sparse) result
  [nonzeros.output-class].
- **FR-017-03** The output MUST contain the nonzero elements of the input in
  column-major order, for both full and sparse inputs, reproducing all four
  observed cases exactly [nonzeros.column-major-order].
- **FR-017-04** (documented choice) An input with no nonzero elements yields
  a 0×1 empty double. The all-zero case is unobserved; this choice follows
  from the n×1 column-vector shape rule with n = 0 and MUST NOT be asserted
  as MATLAB-conformant.
- **FR-017-05** (documented choice) Input kinds beyond the observed dense
  double and sparse arrays MUST be handled by documented independent choice
  and MUST NOT be asserted as MATLAB-conformant (see Unresolved behaviour):
  real scalar-like values (double/int/logical scalars) are treated as 1×1
  arrays; logical and integer-dtype arrays return double values; complex
  inputs keep complex values; NaN counts as nonzero; negative zero counts as
  zero; N-D arrays traverse column-major storage order; GPU tensors are
  gathered to host first; non-numeric kinds (char, string, cell, struct, …)
  are rejected with the stable identifier `RunMat:nonzeros:InvalidInput`.

### Key Entities

- Input: `Value::Tensor` (dense, column-major data) or `Value::SparseTensor`
  (CSC: `col_ptrs`/`row_indices`/`values`; values are stored column-by-column
  with rows sorted, i.e. already in column-major order), plus the
  documented-choice kinds above.
- Output: `Value::Tensor` with shape `[n, 1]` (`Value::Num` when n = 1,
  RunMat's 1×1 double; `Value::Complex`/`Value::ComplexTensor` for the
  documented-choice complex path).

## Success Criteria *(mandatory)*

- **SC-017-1**: Every observed case in the export reproduces exactly (value,
  class double, size n×1) in normative tests.
- **SC-017-2**: The builtin is registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime` passes for its
  test module.
- **SC-017-3**: Traceability: each FR cites ≥1 claim id (or is marked as a
  documented choice); each normative test cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

The export records no unresolved questions
(`unresolved_questions: []`), but the four observations cover only dense
double and sparse double inputs with at least one nonzero. The following
implementation-side choices are therefore documented, tested under the
`unresolved_choice_*` tier, and not asserted as MATLAB-conformant:

- Empty result: 0×1 empty double (FR-017-04).
- Scalars (double/int/logical): treated as 1×1 arrays; nonzero scalar →
  1×1 double, zero scalar → 0×1 empty double.
- Logical and integer-dtype arrays: accepted; output is double (consistent
  with nonzeros.output-class on observed inputs).
- Complex inputs: element nonzero when either part is nonzero or NaN;
  complex values preserved in the output; empty complex result → 0×1 empty
  double.
- NaN elements count as nonzero; negative zero counts as zero.
- N-D arrays: traversed in column-major storage order.
- GPU tensors: gathered to host, then treated as dense arrays.
- Sparse storage containing an explicitly stored zero (constructible via
  `SparseTensor::new`): the zero is skipped.
- Non-numeric inputs (char, string, cell, struct, …): rejected with
  `RunMat:nonzeros:InvalidInput`.

## Assumptions

- RunMat's `SparseTensor` invariants (columns in order, rows sorted and
  unique within each column) make its `values` array column-major, so no
  reordering is needed for the sparse path.
- `Value::Num` is RunMat's canonical 1×1 double, so returning it for a
  single nonzero matches the observed 1×1 result (house convention:
  `tensor_into_value`).
