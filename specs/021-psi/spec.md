# Feature Specification: `psi` — Digamma Function

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `021-psi`;
export `SPECIFY_FEATURE=021-psi`)

**Created**: 2026-07-22

**Status**: Tier B batch 1 — gates approved; full pipeline (docs + implementation)

**Input**: Approved Tier B export `psi` 0.2.0 (`status: approved`; see
`provenance.md` for the commit pin and approval record).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Evaluate the digamma function (Priority: P1)

RunMat users calling `psi(X)` on real numeric input get the digamma function
(the logarithmic derivative of the gamma function, d/dx ln Γ(x)) evaluated
element-wise, as class double, matching the approved observed behaviour.

**Independent Test**: run each observed case from the export through the
builtin and compare value (to floating-point tolerance), class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** the scalar `1`, **When** `psi` is called, **Then** the result is
   a 1×1 double equal to `-0.57721566490153231` (the negative of the
   Euler–Mascheroni constant) within `1e-12`.
   [psi.digamma-values, psi.output-class]
2. **Given** the row vector `[1 2 3]`, **When** `psi` is called, **Then** the
   result is a 1×3 double equal to
   `[-0.57721566490153231 0.42278433509846747 0.92278433509846747]`
   element-wise within `1e-12`. [psi.digamma-values, psi.output-class]

### User Story 2 - Composition with RunMat value kinds (Priority: P2)

Tensors preserve shape; integer and logical inputs promote to double before
evaluation (RunMat convention; unobserved in the export and therefore a
documented independent choice).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-021-01** `psi` MUST accept the single-argument form `Y = psi(X)`
  [psi.signature-primary] and return class double
  [psi.output-class, psi.out-type, psi.output-class-rule].
- **FR-021-02** The result MUST be the digamma function of the input,
  element-wise; `psi(1)` equals the negative of the Euler–Mascheroni
  constant, and the observed scalar and vector cases MUST reproduce within
  `1e-12` [psi.digamma-values, psi.digamma-values-rule].
- **FR-021-03** Shape MUST be preserved: a scalar yields a 1×1 result, the
  observed 1×3 vector yields a 1×3 result (observed sizes in the export's
  `scalar` and `vector` cases).
- **FR-021-04** Inputs not covered by the export — poles at non-positive
  integers [psi.q-poles], negative non-integers, complex values, `NaN`/`Inf`,
  non-double numerics, char/logical inputs, GPU arrays — MUST be handled by
  documented independent choice (see Unresolved behaviour), tagged
  `unresolved_choice_*` in tests, and MUST NOT be asserted as
  MATLAB-conformant.
- **FR-021-05** The two-argument polygamma form `psi(k, X)` is pending in the
  approved specification [psi.q-polygamma] and MUST be rejected with a stable
  error identifier rather than silently mis-evaluated.

### Key Entities

- Scalar result: `Value::Num` (RunMat's double scalar); array result:
  `Value::Tensor` (double), shape preserved.
- Algorithm: independent public-domain evaluation (argument-shift recurrence
  plus Bernoulli asymptotic series); source recorded in `provenance.md`
  under Principle X. No MATLAB internals are inferred or imitated.

## Success Criteria *(mandatory)*

- **SC-021-1**: Both observed cases in the export reproduce (value within
  `1e-12`, class double, sizes 1×1 and 1×3) in normative tests tagged with
  the applicable release (R2026a).
- **SC-021-2**: The builtin is registered and discoverable through the
  standard `#[runtime_builtin]` inventory;
  `cargo test -p runmat-runtime --lib -- builtins::math::elementwise::psi`
  passes.
- **SC-021-3**: Traceability: each FR cites ≥1 claim id or unresolved
  question; each normative test cites ≥1 FR/claim.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `psi.q-poles`: behaviour at non-positive integers (poles of digamma) is
  unobserved. Choice: return `NaN` at exact non-positive integers (the
  two-sided limits diverge with opposite signs, so no principled sign
  exists); values near poles evaluate to large finite magnitudes.
- `psi.q-polygamma`: the two-argument polygamma form is unspecified. Choice:
  any two-argument call errors with `RunMat:psi:PolygammaUnsupported`; more
  than two arguments errors with `RunMat:psi:InvalidArgument`.
- Negative non-integer inputs: unobserved. Choice: evaluate via the same
  recurrence (mathematically defined there); tagged `unresolved_choice_*`.
- Complex inputs: unobserved. Choice: rejected with
  `RunMat:psi:InvalidInput`.
- `NaN` → `NaN`; `+Inf` → `+Inf` (digamma limit); `-Inf` → `NaN`. All
  independent choices.
- Integer/logical/char inputs promote to double (RunMat convention shared
  with `gamma`); GPU tensors gather to host (no provider hook exists for
  psi) and evaluate on the CPU.

## Assumptions

- RunMat's elementwise math conventions (exemplar:
  `crates/runmat-runtime/src/builtins/math/elementwise/gamma.rs`) apply; no
  new runtime types or architecture are required.
- Double-precision evaluation to ≤1e-12 of the observed values is achievable
  with the recorded public-domain algorithm (verified in validation.md).
