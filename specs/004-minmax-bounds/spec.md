# Feature Specification: Min/Max Bounds — `bounds`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `004-minmax-bounds`; export `SPECIFY_FEATURE=004-minmax-bounds`)

**Created**: 2026-07-22

**Status**: Batch 1 — combined review pending

**Input**: Approved Tier A export `bounds` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing

### User Story 1 - Get smallest and largest elements (Priority: P1)

`[lo, hi] = bounds(A)` returns the smallest and largest elements; with a
dimension argument, reduction is along that dimension.

**Acceptance Scenarios** (observed, normative — first output only):

1. **Given** a double vector (case `vector`), **When** `bounds(A)` is called,
   **Then** output 1 is the scalar minimum, class double.
   [bounds.first-output-min, bounds.output-class]
2. **Given** a 2×2-shaped matrix (case `matrix-default`), **Then** output 1
   is the 1×2 column-wise minimum. [bounds.first-output-min]
3. **Given** dim = 2 (case `matrix-dim2`), **Then** output 1 is the m×1
   row-wise minimum. [bounds.first-output-min]

## Requirements

- **FR-004-01** `bounds` MUST support `[lo, hi] = bounds(A)`
  [bounds.signature-primary]; `bounds(A, dim)` is exercised for output 1 by
  observation `matrix-dim2` and MUST be accepted.
- **FR-004-02** Output class MUST be double for double input
  [bounds.output-class].
- **FR-004-03** Output 1 MUST be the minimum with observed shape semantics:
  vector → scalar; matrix default → column-wise 1×n; dim=2 → row-wise m×1
  [bounds.first-output-min].
- **FR-004-04** (summary-derived) Output 2 SHOULD be the largest elements
  with the same shape semantics, per the approved summary ("smallest and
  largest elements"); unobserved (`bounds.q-second-output`), tests tagged
  `summary_derived`.
- **FR-004-05** NaN handling and `'omitnan'`/`'includenan'` options are
  unresolved (`bounds.q-nan`) and MUST NOT be implemented as options in this
  feature; NaN inputs follow whatever RunMat's existing `min`/`max` kernels
  do, documented and tagged, without conformance claims.

## Success Criteria

- **SC-004-1**: The three observed cases reproduce exactly (value, class,
  shape) in normative tests.
- **SC-004-2**: Two-output requests return (lo, hi) via
  `ByRequestedOutputCount`; single-output returns lo.
- **SC-004-3**: Full FR→test traceability.

## Unresolved behaviour (implemented by documented choice)

- `bounds.q-second-output`: hi = maximum with matching shape (summary-derived).
- `bounds.q-nan`: no option arguments; inherited kernel NaN semantics
  documented in code and tagged in tests.
- Non-double numeric classes, empty input, N-D input: unobserved; behaviour
  delegated to the shared reduction machinery, tagged.

## Assumptions

- Implemented over the same reduction machinery as `min`/`max` in
  `math/reduction` (shared kernels; no new algorithm), keeping observable
  shape semantics identical to the observed cases.
