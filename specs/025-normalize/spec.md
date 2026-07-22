# Feature Specification: `normalize`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`025-normalize`; export `SPECIFY_FEATURE=025-normalize`)

**Created**: 2026-07-22

**Status**: Batch 4 (Tier B batch 2) — implementation

**Input**: Approved Tier B export `normalize` 0.2.0, pinned to
matlab-interface-spec approval commit `4445294` (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Standardize data with the default z-score (Priority: P1)

A RunMat user calls `normalize(A)` and gets `A` centred to zero mean and
scaled to unit sample standard deviation, matching the approved observed
behaviour.

**Independent Test**: run the observed case `zscore` through the builtin and
compare value, class and shape: `normalize([1 2 3])` → `[-1 0 1]` (1×3
double).

**Acceptance Scenarios** (from observed results, normative):

1. **Given** the row vector `[1 2 3]`, **When** `normalize` is called with no
   method, **Then** the result is `[-1 0 1]`, class `double`, size 1×3
   (mean 2, **sample** std `sqrt(((1-2)^2+(2-2)^2+(3-2)^2)/(3-1)) = 1`).
   [normalize.signature-primary, normalize.output-class,
   normalize.zscore-and-range]

### User Story 2 - Rescale data to the unit interval (Priority: P1)

`normalize(A, 'range')` rescales the data to `[0, 1]`.

**Acceptance Scenarios** (observed, normative):

1. **Given** `[1 2 3]`, **When** `normalize([1 2 3], 'range')` is called,
   **Then** the result is `[0 0.5 1]`, class `double`, size 1×3
   ((x − min)/(max − min)). [normalize.zscore-and-range,
   normalize.output-class]

### User Story 3 - Other methods, dimensions, intervals (Priority: P3)

Operating on matrices along a chosen dimension, and the `center`, `scale`,
`norm` methods plus custom range intervals, are **unobserved** in the
approved export. RunMat implements them by documented independent choice and
does not assert MATLAB conformance (see Unresolved behaviour).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-025-01** `normalize` MUST accept the primary form `N = normalize(A)`
  [normalize.signature-primary] and return class `double`
  [normalize.output-class] with the same size as `A`.
- **FR-025-02** The default (and explicit `'zscore'`) method MUST centre by
  the mean and scale by the **sample** standard deviation (N−1 denominator),
  reproducing `normalize([1 2 3]) = [-1 0 1]` exactly.
  [normalize.zscore-and-range]
- **FR-025-03** The `'range'` method MUST rescale the operating dimension to
  `[0, 1]` via `(x − min)/(max − min)`, reproducing
  `normalize([1 2 3], 'range') = [0 0.5 1]` exactly.
  [normalize.zscore-and-range]
- **FR-025-04** (documented choice) For vectors the operation covers all
  elements; for matrices/N-D arrays without an explicit dimension it operates
  along the first non-singleton dimension; an explicit numeric dimension
  argument `normalize(A, dim, ...)` selects the operating dimension. Unobserved
  — normalize.q-dim. MUST NOT be asserted MATLAB-conformant.
- **FR-025-05** (documented choice) Additional methods are implemented where
  their semantics are unambiguous: `'center'` (subtract mean, or a numeric
  centre), `'scale'` (divide by sample std, `'first'`, or a numeric scale),
  `'norm'` (divide by the vector p-norm, default p = 2, `Inf` supported), and
  `'range'` with a custom interval `[a b]`. Unobserved — normalize.q-methods.
- **FR-025-06** Methods whose behaviour is unresolved in the approved export
  (`'robust'`, `'medianiqr'`, `'mad'`, `'iqr'`, and `'zscore','robust'`) and
  complex inputs MUST be rejected with a stable error rather than invented.
- **FR-025-07** Inputs are accepted as real double data: `double`, integer,
  logical, and GPU arrays are gathered/converted to host double; the result is
  a real double `Value` (scalar collapses to `Value::Num`).

### Key Entities

- Result: real double `Value` (`Value::Tensor`, or `Value::Num` when scalar),
  same size as the input.
- Reduction statistics (mean, sample std, min, max, p-norm) computed along the
  operating dimension and broadcast back across it.

## Success Criteria *(mandatory)*

- **SC-025-1**: The two observed cases (`zscore`, `range`) reproduce exactly
  (value, class double, size 1×3) in normative tests.
- **SC-025-2**: `normalize` is registered and discoverable via the builtin
  registry; `cargo test -p runmat-runtime` passes for its test module.
- **SC-025-3**: Traceability — each FR cites ≥1 claim id or unresolved
  question; each normative test cites ≥1 FR/claim; unobserved tests are tagged
  `unresolved_choice_*`.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `normalize.q-dim`: operating dimension for matrices. Choice: first
  non-singleton dimension by default; explicit numeric `dim` argument honoured;
  a `dim` beyond the array rank selects a singleton axis (each element on its
  own → z-score NaN).
- `normalize.q-methods`: other methods (`norm`, `center`, `scale`,
  `medianiqr`, robust variants). Choices: `center`/`scale`/`norm` implemented
  as above; robust/median/MAD/IQR variants **not** invented — rejected with
  `RM.NORMALIZE.UNSUPPORTED`.
- NaN handling and degenerate scales (constant vector → std 0, single element)
  are unobserved. Choice: statistics propagate NaN for z-score/center/scale;
  min/max ignore NaN; degenerate divisions yield NaN (0/0). Documented, not
  asserted conformant.
- Complex inputs: unobserved. Choice: rejected (`RM.NORMALIZE.UNSUPPORTED`).

## Assumptions

- RunMat's value model provides real double tensors and host/GPU gather
  helpers used by the sibling reduction builtins (`mean`, `std`); no new
  runtime types are required.
- Output class `double` (observed) is honoured by returning a real double
  value regardless of input numeric class.
