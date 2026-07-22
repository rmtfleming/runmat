# Feature Specification: Time Conversion — `etime`, `seconds`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`029-time-conversion`; export `SPECIFY_FEATURE=029-time-conversion`)

**Created**: 2026-07-22

**Status**: Batch 5 (Tier B batch 3) — combined review pending

**Input**: Approved Tier B exports `etime` 0.2.0, `seconds` 0.2.0 (see
`provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Elapsed seconds between two date vectors (Priority: P1)

RunMat users calling `etime(t2, t1)` on two six-element date vectors
`[Y M D H MN S]` get the number of seconds elapsed from `t1` to `t2` as a
`double` scalar, matching the approved observed behaviour.

**Independent Test**: run the observed case from the export through the
builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** two date vectors one second apart, **When** `etime` is called,
   **Then** the result is the `double` scalar `1` (class double, size 1×1).
   [etime.signature-primary, etime.output-class, etime.elapsed-seconds]

### User Story 2 - Build a duration from a number of seconds (Priority: P1)

`seconds(x)` returns a `duration` value representing `x` seconds, using
RunMat's existing `duration` representation.

**Acceptance Scenarios**:

1. **Given** the number `5`, **When** `seconds` is called, **Then** the result
   has class `duration`, size 1×1, and magnitude 5 seconds.
   [seconds.signature-primary, seconds.output-class, seconds.to-duration]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-029-01** `etime` MUST accept two arguments (`s = etime(t2, t1)`)
  [etime.signature-primary] and return class `double`, 1×1
  [etime.output-class], equal to the number of seconds between the two
  date vectors (`1` for vectors one second apart) [etime.elapsed-seconds].
- **FR-029-02** `seconds` MUST accept one argument (`d = seconds(x)`)
  [seconds.signature-primary] and return class `duration`, 1×1 for scalar
  input [seconds.output-class].
- **FR-029-03** `seconds(5)` MUST return a `duration` object whose magnitude
  is 5 seconds [seconds.to-duration], built with RunMat's existing
  `duration` (days-based) representation.
- **FR-029-04** (unresolved) `etime` behaviour when `t2` precedes `t1`
  (`etime.q-negative`), across day/month/year boundaries, and for fractional
  seconds is unobserved; it MUST be handled by documented independent choice
  and MUST NOT be asserted as MATLAB-conformant.
- **FR-029-05** (unresolved) `etime` inputs that are not six-element finite
  date vectors, and wrong argument counts, MUST be rejected by documented
  independent choice (clear error), not guessed.
- **FR-029-06** (unresolved) `seconds` array inputs, the duration-to-number
  inverse form (`seconds(d)`, `seconds.q-from-duration`), the chosen display
  `Format`, and non-numeric inputs MUST be handled by documented independent
  choice and MUST NOT be asserted as MATLAB-conformant.

### Key Entities

- `etime` result: `Value::Num` (RunMat's scalar double).
- `seconds` result: a `duration` object built via the duration module's
  internal days-based representation (`__days` property), the same
  representation the `duration` constructor produces.

## Success Criteria *(mandatory)*

- **SC-029-1**: Both observed cases (`etime` one-second → `1`; `seconds(5)` →
  a 1×1 `duration` of 5 seconds) reproduce exactly (value, class, size) in
  normative tests.
- **SC-029-2**: Both builtins registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime` passes for their
  test modules.
- **SC-029-3**: Traceability: each FR cites ≥1 claim id; each normative test
  cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `etime.q-negative`: result when `t2 < t1`. Choice: the difference
  `seconds(t2) − seconds(t1)`, which is negative when `t2` precedes `t1`.
- `etime.q-deprecated`: deprecation notice unobserved. Choice: none emitted.
- `etime` non-six-element / non-numeric / non-finite inputs and wrong argument
  counts: unobserved. Choice: rejected with a clear `RunMat:etime:InvalidInput`
  error.
- `seconds.q-from-duration`: the duration-to-number form `seconds(d)` is
  unobserved. Choice: non-numeric inputs (including duration values) are
  rejected with a clear `RunMat:seconds:InvalidInput` error.
- `seconds` array inputs: unobserved. Choice: mapped element-wise to a
  duration array of the same shape.
- `seconds` display `Format`: the observed value is opaque
  (`<duration [1 1]>`). Choice: `"s"` (render the value in seconds).

## Assumptions

- RunMat's `duration` value model (module
  `crates/runmat-runtime/src/builtins/duration`) already stores durations as a
  days-valued tensor; `seconds` reuses `duration_object_from_days_tensor`, no
  new runtime type is required.
- The elapsed-seconds computation is epoch-independent: `etime` returns a
  difference, so any consistent civil-to-days conversion reproduces the
  observed result.
