# Feature Specification: `histc` — legacy edge-based histogram counts

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`024-histc`; export `SPECIFY_FEATURE=024-histc`)

**Created**: 2026-07-22

**Status**: Batch 4 (Tier B batch 2) — review approved 2026-07-22

**Input**: Approved Tier B export `histc` 0.2.0 (`status: approved`), pinned to
source commit `4445294` (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Count values into edge-defined bins (Priority: P1)

RunMat users calling `n = histc(x, edges)` get a `double` vector of per-bin
counts. Bin `k` counts values `v` with `edges(k) <= v < edges(k+1)`; the final
bin counts values exactly equal to `edges(end)`. The output has one element per
edge (length `length(edges)`).

**Independent Test**: run the single approved observed case through the builtin
and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** `x = [1 2 2 3]` and `edges = [1 2 3]`, **When** `histc(x, edges)`
   is called, **Then** the result is the `1x3` `double` row `[1 2 1]`:
   bin 1 `[1,2)` = {1} = 1, bin 2 `[2,3)` = {2,2} = 2, and the final bin
   (values equal to the last edge `3`) = {3} = 1.
   [histc.signature-primary, histc.output-class, histc.bin-counts]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-024-01** `histc` MUST accept the primary form `n = histc(x, edges)`
  [histc.signature-primary] and return class `double` [histc.output-class].
- **FR-024-02** For the observed input `histc([1 2 2 3], [1 2 3])`, the result
  MUST equal `[1 2 1]` with size `1x3` [histc.bin-counts].
- **FR-024-03** (normative rule, backed by the observed case) Each output
  element `n(k)` MUST count values `v` with `edges(k) <= v < edges(k+1)` for
  `k = 1 .. length(edges)-1`; the final element `n(length(edges))` MUST count
  values exactly equal to `edges(end)`. Output length MUST equal
  `length(edges)` [histc.bin-counts].
- **FR-024-04** (documented independent choice; unobserved) Values below
  `edges(1)`, values strictly greater than `edges(end)`, and `NaN` values MUST
  NOT be counted; the count vector's orientation MUST follow the operating
  dimension (row input → row output, column input → column output). These are
  independent inferences and MUST NOT be asserted as MATLAB-conformant.
- **FR-024-05** (documented independent choice; unobserved) For matrix and
  N-D inputs the counts MUST be computed independently along the first
  non-singleton dimension by default, or along the dimension given by an
  optional third argument `histc(x, edges, dim)`; the operating dimension's
  size becomes `length(edges)` and all other dimensions are preserved.
- **FR-024-06** (documented independent choice; unobserved) A requested second
  output `[n, idx] = histc(x, edges)` MUST return `idx` with the same size as
  `x`, giving the 1-based bin index each element falls into, or `0` for
  uncounted elements.
- **FR-024-07** Inputs and options not covered by the export (edge
  monotonicity errors, GPU values, second output, `dim`) MUST be handled by
  documented independent choice and MUST NOT be asserted as MATLAB-conformant.

### Key Entities

- Counts result: `Value::Tensor` of `f64` (RunMat's `double`), oriented along
  the operating dimension.
- Bin-index result (second output): `Value::Tensor` with the same shape as the
  input.

## Success Criteria *(mandatory)*

- **SC-024-1**: The single observed case reproduces exactly (value `[1 2 1]`,
  class double, size `1x3`) in a normative test.
- **SC-024-2**: `histc` is registered and discoverable via
  `builtin_function_by_name`; the `histc` test module passes under
  `cargo test -p runmat-runtime`.
- **SC-024-3**: Traceability: each FR cites ≥1 claim id (or is marked a
  documented choice); each normative test cites ≥1 FR/claim id.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `histc.q-deprecated`: any deprecation notice relative to `histcounts` is
  unconfirmed; RunMat implements `histc` as an ordinary builtin with no warning.
- Orientation of the output for a column vector, matrix column-wise behaviour,
  the `dim` argument, the second output (bin indices), values outside
  `[edges(1), edges(end)]`, `NaN` handling, and edge-monotonicity error
  behaviour are all unobserved. Choices are documented in FR-024-04..07 and
  exercised by `unresolved_choice_*` tests without conformance claims.

## Assumptions

- RunMat's `Value`/`Tensor` model represents `double` arrays column-major;
  `histc` reuses it and the shared `tensor`/`gpu_helpers` conversion helpers
  used by `histcounts`.
- No new runtime types are required.
