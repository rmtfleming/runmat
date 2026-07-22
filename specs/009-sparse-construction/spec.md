# Feature Specification: Sparse Construction — `full`, `speye`, `spdiags`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `009-sparse-construction`; export `SPECIFY_FEATURE=009-sparse-construction`)

**Created**: 2026-07-22

**Status**: Implemented — validation recorded (`validation.md`); human review pending

**Input**: Approved Tier A exports `full` 0.2.0, `speye` 0.2.0,
`spdiags` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Convert sparse storage to full storage (Priority: P1)

RunMat users calling `full(S)` on a sparse matrix get the dense double array
with the same values and shape; an already-full input is returned in full
storage, matching the approved observed behaviour.

**Independent Test**: run each observed case from the export through the
builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** the sparse form of `[1 0;0 2]`, **When** `full` is called,
   **Then** the result is the 2×2 dense double `[1 0;0 2]` (issparse=false).
   [full.full-preserves-values, full.output-class]
2. **Given** the sparse form of `[0 3 0]`, **When** `full` is called,
   **Then** the result is the 1×3 dense double `[0 3 0]`.
   [full.full-preserves-values]
3. **Given** the dense double `[1 2;3 4]`, **When** `full` is called,
   **Then** the result is the same 2×2 dense double, unchanged.
   [full.full-preserves-values]
4. **Given** an empty sparse matrix, **When** `full` is called, **Then** the
   result is a 0×0 dense double. [full.full-preserves-values]

### User Story 2 - Create a sparse identity matrix (Priority: P1)

`speye(n)`, `speye(m, n)` and `speye([m n])` return a sparse double matrix
(issparse=true) with ones on the main diagonal and zeros elsewhere.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** `speye(3)`, **Then** the result is the 3×3 sparse identity
   `sparse([1;2;3], [1;2;3], [1;1;1], 3, 3)`. [speye.signature-primary,
   speye.sparse-identity, speye.output-class]
2. **Given** `speye(2, 4)`, **Then** the result is the 2×4 sparse matrix
   `sparse([1;2], [1;2], [1;1], 2, 4)`. [speye.sparse-identity]
3. **Given** `speye([3 3])`, **Then** the result equals `speye(3)`.
   [speye.sparse-identity]
4. **Given** `speye(0)`, **Then** the result is the 0×0 sparse matrix
   `sparse([], [], [], 0, 0)`. [speye.sparse-identity]

### User Story 3 - Extract or construct matrix diagonals (Priority: P2)

`spdiags(A)` returns the nonzero diagonals of `A` as the columns of a full
matrix; `spdiags(B, d, m, n)` builds an m×n sparse matrix from the columns
of `B` placed on the diagonals `d`.

**Acceptance Scenarios**:

1. **Given** `spdiags([1 2 0; 0 3 4; 0 0 5])`, **Then** the result is the
   3×2 dense double `[1 0;3 2;5 4]` (issparse=false).
   [spdiags.signature-primary, spdiags.extract-vs-construct,
   spdiags.output-class]
2. **Given** `spdiags([1; 2; 3], 0, 3, 3)`, **Then** the result is the 3×3
   sparse matrix `sparse([1;2;3], [1;2;3], [1;2;3], 3, 3)` (issparse=true).
   [spdiags.extract-vs-construct]
3. **Given** `[B, d] = spdiags(A)`, **Then** `d` is the column vector of the
   extracted diagonal indices. *(Summary-derived: the multi-output
   extraction form is unresolved in the approved spec — see Unresolved.
   Test tagged `summary_derived`.)*

## Requirements *(mandatory)*

### Functional Requirements

- **FR-009-01** `full` MUST accept one argument (`B = full(S)`)
  [full.signature-primary]; for a sparse input it MUST return the dense
  double array (class double, issparse=false) with the same values and
  shape, expanding in column-major order; an empty sparse input MUST yield
  a 0×0 dense double [full.output-class, full.full-preserves-values].
- **FR-009-02** `full` MUST return an already-full input unchanged (observed
  for a dense double matrix) [full.full-preserves-values].
- **FR-009-03** `full` inputs not covered by the export (numeric scalars,
  logicals, GPU tensors, non-numeric kinds, complex data) MUST be handled by
  documented independent choice (non-sparse values pass through unchanged)
  and MUST NOT be asserted as MATLAB-conformant.
- **FR-009-04** `speye` MUST accept `speye(n)`, `speye(m, n)` and
  `speye([m n])` [speye.signature-primary] and return a sparse double matrix
  (issparse=true) with ones at positions (i, i) for i ≤ min(m, n) and zeros
  elsewhere; `speye(0)` MUST yield the 0×0 sparse matrix
  [speye.output-class, speye.sparse-identity].
- **FR-009-05** The `speye` result MUST use RunMat's sparse representation
  (`Value::SparseTensor`, class double), never a dense tensor
  [speye.output-class, speye.sparse-identity].
- **FR-009-06** `speye` inputs not covered by the export (non-integer,
  negative or oversized dimensions; size vectors with other than two
  elements; more than two arguments) MUST be handled by documented
  independent choice (rejected with `RunMat:speye:InvalidArgument`) and MUST
  NOT be asserted as MATLAB-conformant.
- **FR-009-07** `spdiags` MUST accept the extraction form `B = spdiags(A)`
  [spdiags.signature-primary] and return the nonzero diagonals of `A` as the
  columns of a full (dense double) matrix with min(m, n) rows, reproducing
  the observed layout exactly: for the observed 3×3 input the main diagonal
  `[1;3;5]` is column 1 and superdiagonal 1 is `[0;2;4]` (zero-padded at the
  top) [spdiags.extract-vs-construct, spdiags.output-class]. Sparse and full
  inputs use the same layout (summary text; the observed case used a full
  input).
- **FR-009-08** `spdiags` MUST accept the construction form
  `A = spdiags(B, d, m, n)` and return an m×n sparse matrix (issparse=true)
  with column k of `B` placed on diagonal d(k); the observed case
  `spdiags([1;2;3], 0, 3, 3)` MUST reproduce exactly
  [spdiags.extract-vs-construct].
- **FR-009-09** (summary-derived) The second extraction output `d` SHOULD be
  the column vector of extracted diagonal indices, ascending; no observation
  case backs this yet (spdiags.q-outputs).
- **FR-009-10** Diagonal combinations and call forms not covered by the
  export MUST be handled by documented independent choice (see Unresolved
  behaviour) and MUST NOT be asserted as MATLAB-conformant. The implemented
  general rule is: element A(i, j) lies on diagonal d = j − i (d < 0
  subdiagonals, d > 0 superdiagonals); extracted diagonals are sorted
  ascending; for m ≥ n the element A(i, j) is stored in row j of B
  (superdiagonals zero-padded at the top, subdiagonals at the bottom), for
  m < n in row i (opposite padding); construction applies the inverse
  mapping, ignoring out-of-range positions. This rule reproduces the
  observed extract and construct cases exactly.

### Key Entities

- Sparse values: `Value::SparseTensor` (RunMat's compressed-sparse-column
  matrix, real f64); dense results: `Value::Tensor` (column-major, class
  double).
- No `sparse`-builtin dependency: `full`, `speye` and `spdiags` construct
  and consume `SparseTensor` values directly.

## Success Criteria *(mandatory)*

- **SC-009-1**: Every observed case in the three exports (4 + 4 + 2 = 10)
  reproduces exactly (value, class, shape, sparse/dense storage) in
  normative tests.
- **SC-009-2**: All three builtins registered and discoverable via the
  builtin registry; `cargo test -p runmat-runtime` passes for their test
  modules.
- **SC-009-3**: Traceability: each FR cites ≥1 claim id (or is explicitly
  summary-derived/choice); each normative test cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `full.q-class`: output class/issparse over wider input set. Choice:
  non-sparse inputs pass through unchanged (identity); sparse inputs always
  densify to `Value::Tensor`.
- `full.q-complex`: complex sparse input unobserved. RunMat's
  `SparseTensor` is real-valued, so the case cannot arise on this substrate;
  carried unresolved for the specification side.
- `speye.q-class`: class/diagonal confirmation over wider input set.
  Choice: dimensions must be nonnegative integer scalars or a two-element
  size vector; anything else (including zero or more than two arguments) is
  rejected with `RunMat:speye:InvalidArgument`.
- `spdiags.q-forms`: only `spdiags(A)` and `spdiags(B, d, m, n)` are
  implemented; the two-argument extract form and three-argument replace form
  are rejected with `RunMat:spdiags:InvalidArgument`.
- `spdiags.q-outputs`: second output `d` implemented as summary-derived
  (FR-009-09), p×1 double column vector, ascending.
- `spdiags` construction edge cases (unobserved, documented choices): `B`
  must be min(m, n)×length(d) (mirror of the extraction layout); zero values
  are not stored in the result; a repeated diagonal index lets the later
  column win; stored NaN counts as nonzero.

## Assumptions

- RunMat's value model already provides `Value::SparseTensor` (CSC) with
  validated constructors and `to_dense()`; no new runtime types required.
- Observed sparse values recorded in the exports as
  `sparse(i, j, v, m, n)` expressions are translated to the equivalent CSC
  literals when constructing test inputs/expectations.
