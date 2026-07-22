# Feature Specification: Sparse Ordering — `colamd`, `dissect`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`027-sparse-ordering`; export `SPECIFY_FEATURE=027-sparse-ordering`)

**Created**: 2026-07-22

**Status**: Batch 4 (Tier B batch 2) — combined review pending

**Input**: Approved Tier B batch-2 exports `colamd` 0.2.0, `dissect` 0.2.0
(see `provenance.md`).

## Overview

`colamd` and `dissect` return **fill-reducing column permutation vectors** for
a sparse matrix. Both are used to reorder a matrix before factorization so that
the factors have fewer nonzeros. The approved exports confirm only the
*interface and result shape*: each returns a class-`double`, 1×n row vector
that is a permutation of `1..n` (for an n-column / n×n input). **The specific
permutation values are algorithm-dependent and are NOT normative** (see the
"Non-assertion of specific permutations" section below); the exports carry
`colamd.q-values` / `dissect.q-values` as explicitly unresolved.

## Non-assertion of specific permutations *(binding scope decision)*

The observed MATLAB outputs — `colamd(sparse([1 0 1;0 1 0;1 0 1])) = [2 1 3]`
and `dissect(sparse([2 1 0;1 2 1;0 1 2])) = [3 1 2]` — are produced by
MathWorks/SuiteSparse/METIS internal algorithms. Per Constitution Principle III
(no reproduction of MATLAB internal algorithms) and the approved exports (which
state "the specific ordering is algorithm-dependent and is not asserted"), this
feature:

- **Does NOT assert** the observed permutation values as a requirement.
- Implements an **independently-derived** fill-reducing heuristic from public
  algorithm literature (recorded under Principle X in `provenance.md`):
  - `colamd`: classic **minimum-degree** ordering on the column-intersection
    graph (Tinney & Walker 1967; George & Liu 1981).
  - `dissect`: basic **recursive nested dissection** via level-structure graph
    bisection (George 1973; George & Liu 1981; Gibbs–Poole–Stockmeyer 1976).
- Records each implementation's *own* output on the observed input in a
  `documented_choice` test, explicitly flagged non-normative.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Fill-reducing column ordering (Priority: P1)

A RunMat user calls `p = colamd(S)` (or `p = dissect(S)`) on a sparse matrix
and receives a permutation vector `p` (1×n double, a permutation of `1..n`)
suitable for reordering `S` before factorization.

**Independent Test**: for each observed case, run the builtin and assert the
result is (a) class double, (b) size 1×n, (c) a valid permutation of `1..n`
(each index exactly once). The exact values are not asserted.

**Acceptance Scenarios** (from approved shape/class claims, normative):

1. **Given** the observed sparse matrix, **When** `colamd(S)` is called,
   **Then** the result is a class-double 1×3 row vector that is a permutation
   of `{1,2,3}`. [colamd.output-class, colamd.permutation-vector]
2. **Given** the observed symmetric sparse matrix, **When** `dissect(S)` is
   called, **Then** the result is a class-double 1×3 row vector that is a
   permutation of `{1,2,3}`. [dissect.output-class, dissect.permutation-vector]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-027-01** `colamd` and `dissect` MUST each accept one matrix argument
  (`p = colamd(S)` / `p = dissect(S)`) [colamd.signature-primary,
  dissect.signature-primary].
- **FR-027-02** `colamd(S)` MUST return class `double`
  [colamd.output-class], a 1×n row vector that is a permutation of `1..n`
  where n is the number of columns of `S` [colamd.permutation-vector]. The
  ordering is produced by an independent minimum-degree heuristic; the specific
  values are NOT asserted [colamd.q-values].
- **FR-027-03** `dissect(S)` MUST return class `double`
  [dissect.output-class], a 1×n row vector that is a permutation of `1..n`
  [dissect.permutation-vector]. The ordering is produced by an independent
  nested-dissection heuristic; the specific values are NOT asserted
  [dissect.q-values].
- **FR-027-04** (documented choice, non-normative) Each builtin's ordering on
  the observed input, on scalar (1×1) inputs, and the empty-matrix →
  empty-permutation (1×0) behaviour are documented choices, recorded in tests
  tagged `documented_choice`; they are not asserted to match MATLAB.
- **FR-027-05** (documented choice) Inputs not covered by the export — dense
  matrices, logical matrices, non-square matrices for `dissect`, N-D arrays,
  and non-matrix values — MUST be handled by documented independent choice:
  dense/logical are read as structural patterns; a non-square `dissect` input
  is symmetrised via the column-intersection graph (`A+Aᵀ` is undefined for
  rectangular inputs); N-D and non-matrix inputs are rejected with a stable
  `InvalidInput` identifier. None of these are asserted as MATLAB-conformant.

### Key Entities

- Permutation result: `Value::Tensor` of shape `[1, n]` holding one-based
  indices as `f64` (RunMat's 1×n double row vector). Empty input → `[1, 0]`.
- Input pattern: the set of structurally nonzero `(row, col)` coordinates read
  from `Value::SparseTensor` (CSC) or dense/logical storage (column-major).

## Success Criteria *(mandatory)*

- **SC-027-1**: For each observed case, the normative test confirms class
  double, size 1×n, and a valid permutation of `1..n`.
- **SC-027-2**: Both builtins are registered and discoverable via
  `builtin_function_by_name`; the targeted `cargo test` modules pass.
- **SC-027-3**: Traceability: each FR cites ≥1 claim id; each normative test
  cites ≥1 FR/claim. The non-assertion of specific permutation values is
  explicit in `spec.md`, `validation.md`, and the test comments.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `colamd.q-permutation` / `dissect.q-permutation`: orientation/permutation
  confirmation — satisfied by the normative valid-permutation assertion; wider
  input confirmation remains spec-side.
- `colamd.q-values` / `dissect.q-values`: **the specific permutation values are
  algorithm-dependent and are treated as non-normative** (carried unresolved;
  this feature deliberately does not assert them).
- Unobserved input classes (dense, logical, non-square `dissect`, N-D,
  non-matrix, explicit stored zeros, NaN entries) — documented choices, tested
  under the `documented_choice` tier; candidates for future spec-side
  observation.

## Assumptions

- RunMat's `SparseTensor` (CSC) and `Tensor` value models suffice; no new
  runtime type is required.
- The permutation length `n` is the number of columns of the input (equal to
  the number of rows for the observed square inputs).
