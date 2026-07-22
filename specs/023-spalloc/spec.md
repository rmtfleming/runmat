# Feature Specification: Sparse Preallocation — `spalloc`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `023-spalloc`; export `SPECIFY_FEATURE=023-spalloc`)

**Created**: 2026-07-22

**Status**: Batch 4 (Tier B batch 2) — review pending

**Input**: Approved Tier B export `spalloc` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Preallocate an all-zero sparse matrix (Priority: P1)

RunMat users calling `S = spalloc(m, n, nz)` get an all-zero `m`-by-`n` sparse
matrix (class `double`, `issparse` true) with no stored nonzeros, matching the
approved observed behaviour. `nz` states the intended nonzero capacity but does
not change the returned content.

**Independent Test**: run the observed export case through the builtin and
compare value, class, size, and stored-nonzero count.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** `m = 3`, `n = 3`, `nz = 4`, **When** `spalloc(3, 3, 4)` is called,
   **Then** the result is a `3`-by-`3` sparse matrix of class `double`
   (`issparse` true) with no stored nonzeros (`nnz == 0`).
   [spalloc.signature-primary, spalloc.output-class, spalloc.all-zero-sparse]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-023-01** `spalloc` MUST accept the three-argument form
  `S = spalloc(m, n, nz)` [spalloc.signature-primary].
- **FR-023-02** The output MUST have class `double` (RunMat sparse `double`)
  [spalloc.output-class].
- **FR-023-03** The output MUST be an all-zero sparse matrix of the requested
  size: `issparse` true, size `[m, n]`, and no stored nonzeros (`nnz == 0`)
  [spalloc.all-zero-sparse].
- **FR-023-04** The `nz` argument is a capacity hint only and MUST NOT change
  the observable content: `nnz` remains `0` regardless of `nz`. RunMat's CSC
  representation has no separate reserved capacity, so the hint is accepted and
  ignored for content purposes (documented independent choice; the export
  observed only `nz = 4`) [spalloc.q-nz-capacity].
- **FR-023-05** Inputs not covered by the single observed case (non-integer,
  negative, or non-scalar dimensions; invalid `nz`; other argument counts) MUST
  be handled by documented independent choice (see Unresolved behaviour) and
  MUST NOT be asserted as MATLAB-conformant.

### Key Entities

- Result: `Value::SparseTensor` built via `SparseTensor::zeros(m, n)` —
  compressed-sparse-column form with `col_ptrs` all zero and empty
  `row_indices`/`values` (nnz = 0).

## Success Criteria *(mandatory)*

- **SC-023-1**: The observed case `spalloc(3, 3, 4)` reproduces exactly (value,
  class `double`, size `[3, 3]`, `nnz == 0`) in a normative test.
- **SC-023-2**: `spalloc` is registered and discoverable through the
  `#[runtime_builtin]` inventory; `cargo test -p runmat-runtime` passes for its
  test module.
- **SC-023-3**: Traceability: each FR cites ≥1 claim id; each normative test
  cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

The approved export records **no** unresolved questions and a single observed
case. The following are RunMat-side independent choices, tagged
`unresolved_choice_*` in tests and returned to the specification side:

- `spalloc.q-nz-capacity`: the effect of `nz` beyond the observed value `4` is
  unobserved. Choice: `nz` is validated as a nonnegative integer for consistency
  with the dimensions but does not affect stored content (nnz stays 0).
- `spalloc.q-dims`: only the `3`-by-`3` size was observed. Choice: any
  nonnegative-integer `(m, n)` is accepted, producing the corresponding all-zero
  sparse matrix (rectangular and empty sizes included).
- `spalloc.q-invalid-dims`: non-integer / negative / non-scalar dimensions are
  unobserved. Choice: rejected with `RunMat:spalloc:InvalidArgument`.
- `spalloc.q-arity`: only the three-argument form was observed. Choice: exactly
  three arguments are required; other counts (including a missing `nz`) are
  rejected with `RunMat:spalloc:InvalidArgument`.

## Assumptions

- RunMat's value model already provides `Value::SparseTensor` and
  `SparseTensor::zeros`; no new runtime types are required.
- Dimensions and the capacity hint are parsed with the sparse module's shared
  `scalar_f64` / `size_from_f64` helpers, matching `speye` and the other sparse
  constructors.
