# Feature Specification: `eps` — floating-point relative accuracy (spacing)

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`028-eps-function`; export `SPECIFY_FEATURE=028-eps-function`)

**Created**: 2026-07-22

**Status**: Batch 5 (Tier B batch 3) — implementation complete; gate approved
with the binding constraint below.

**Input**: Approved Tier B export `eps` 0.2.0 (`status: approved`); see
`provenance.md`.

## Gate constraint (binding)

RunMat previously registered `eps` as a **constant**
(`register_constant!("eps", Value::Num(f64::EPSILON), …)`). The approved spec
needs the **function** forms (`eps('single')`, and the documented `eps(x)`
spacing form), which a constant cannot express. The gate decision is:

> Replace the `eps` constant with an `eps` **function**. Remove only the
> `eps` constant registration (leave `pi`, `i`, `j`, `Inf`, `NaN`, … intact),
> add a `#[runtime_builtin(name = "eps", …)]` function, and **verify that bare
> `eps` still resolves as a zero-argument builtin call** in RunMat's VM. If
> bare `eps` would not resolve, STOP and report a blocker instead of shipping
> a broken bare `eps`.

The bare-`eps` verification result is recorded in `plan.md` and
`validation.md` (it resolves correctly — evidence included).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read floating-point precision from MATLAB code (Priority: P1)

RunMat users evaluate `eps` (no arguments) and receive the double spacing near
1.0; `eps('single')` returns the single-precision spacing, as a value of class
`single`. Both reproduce the approved observed behaviour exactly.

**Independent Test**: run each observed case from the export through the
builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** no arguments, **When** `eps` is evaluated, **Then** the result is
   the double scalar `2.2204460492503131e-16` (`f64::EPSILON`), class `double`,
   size `1×1`. [eps.signature-primary, eps.spacing-constants]
2. **Given** the class name `'single'`, **When** `eps('single')` is called,
   **Then** the result is the single scalar `1.1920928955078125e-07`
   (`f32::EPSILON`), class `single`, size `1×1`. [eps.spacing-constants]

### User Story 2 - Spacing at a given value or class (Priority: P2)

`eps(x)` for a numeric `x`, and `eps('double')`, extend `eps` beyond the two
observed cases. These forms are **unresolved** in the approved export
(`eps.q-value-form`: the value form is not exercised) and are implemented by
documented independent choice — never asserted as MATLAB-conformant.

**Acceptance Scenarios** (documented choice, non-normative):

1. **Given** a numeric value `x`, **When** `eps(x)` is called, **Then** the
   result is the positive distance from `|x|` to the next larger
   floating-point number of the same class (IEEE `nextUp`-based spacing),
   elementwise for arrays, with class and shape following the input.
2. **Given** the class name `'double'`, **When** `eps('double')` is called,
   **Then** the result equals bare `eps`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-028-01** Bare `eps` (zero arguments) MUST return `Value::Num(f64::EPSILON)`
  = `2.2204460492503131e-16`, class `double`, size `1×1`.
  [eps.signature-primary, eps.spacing-constants]
- **FR-028-02** `eps('single')` MUST return the single spacing
  `1.1920928955078125e-07` as a value of class `single`, size `1×1`
  (RunMat single scalar = 1×1 `NumericDType::F32` tensor). [eps.spacing-constants]
- **FR-028-03** (documented choice; `eps.q-value-form` unresolved) `eps(x)`
  for numeric `x` MUST return the elementwise IEEE `nextUp` spacing at `|x|`,
  preserving the input's class (`double`/`single`) and shape.
- **FR-028-04** (documented choice) `eps('double')` MUST equal bare `eps`;
  class-name matching is case-insensitive; a class name other than `'double'`
  or `'single'` MUST raise `RunMat:eps:InvalidClass`.
- **FR-028-05** (documented choice) Inputs outside the export's scope
  (negative/complex/array values, integer/logical/gpuArray inputs, more than
  one argument) MUST be handled by documented independent choice (see
  Unresolved behaviour) and MUST NOT be asserted as MATLAB-conformant.
- **FR-028-06** The migration MUST remove **only** the `eps` constant
  registration; every other constant (`pi`, `i`, `j`, `inf`/`Inf`,
  `nan`/`NaN`, `sqrt2`, `true`, `false`) MUST remain registered, and bare
  `eps` MUST continue to evaluate (now as a zero-argument builtin call).

### Key Entities

- Double spacing result: `Value::Num` (RunMat double scalar).
- Single spacing result: 1×1 `Value::Tensor` with `dtype = NumericDType::F32`
  (so `class(...)` reports `single`; a `Value::Num` or a collapsed 1-element
  tensor would report `double`).
- Class-name argument: extracted via the shared `keyword_of` helper
  (`String` / `StringArray` / single-row `CharArray`, lowercased).

## Success Criteria *(mandatory)*

- **SC-028-1**: Both observed cases reproduce exactly (value, class, size) in
  normative tests.
- **SC-028-2**: `eps` is registered and discoverable via
  `builtin_function_by_name("eps")`, and is **absent** from `constants()`;
  bare `eps` resolves end-to-end through the VM.
- **SC-028-3**: Traceability: each FR cites ≥1 claim id or is tagged a
  documented choice; each normative test cites ≥1 FR/claim.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `eps.q-value-form`: the `eps(x)` value-dependent form is not exercised in
  the export. Choice: elementwise IEEE `nextUp` spacing at `|x|`.
  - Edge cases follow `nextUp` directly: `eps(0)` is the smallest positive
    subnormal double; `eps(realmax)` is `Inf`; `eps(Inf)` and `eps(NaN)` are
    `NaN`. Documented, unobserved.
  - Negative `x`: uses `|x|` (spacing is symmetric about 0).
  - Complex `x`: spacing measured at the magnitude `|x|`, returned as a real
    value of the same shape. Documented, unobserved.
- `eps('double')`: unobserved; chosen to equal bare `eps`.
- Integer / logical / gpuArray inputs, and >1 argument: rejected with
  `RunMat:eps:InvalidInput` / `RunMat:eps:TooManyInputs` (documented choice).

## Assumptions

- RunMat represents a single-precision scalar as a 1×1 `NumericDType::F32`
  tensor; `class_name_for_value` maps that to `single`
  (`crates/runmat-runtime/src/builtins/introspection/class.rs`). Verified
  during implementation.
- The HIR lowering resolves a bare identifier that is neither a variable nor a
  constant to a zero-argument builtin call when `is_builtin(name)` is true
  (`crates/runmat-hir/src/lowering/ctx.rs`). Verified during implementation.
