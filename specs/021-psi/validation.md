# Validation Report — 021-psi — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean
- `cargo fmt` then `cargo fmt --all -- --check` — clean
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::elementwise::psi`
  — **18 passed, 0 failed** (re-run after formatting; identical result)

## Numerical accuracy (computed vs observed, actual run output)

Observed values are floating-point black-box observations; the acceptance
tolerance is `1e-12`. The implementation (recurrence + Bernoulli asymptotic
series, shift threshold 10, terms through B14) achieved:

| Case | Computed | Observed (export) | \|delta\| | Tolerance met |
|---|---|---|---|---|
| `psi(1)` | `-5.77215664901532421e-1` | `-0.57721566490153231` | `1.110e-16` | yes (1e-12) |
| `psi(2)` | `4.22784335098467412e-1` | `0.42278433509846747` | `5.551e-17` | yes (1e-12) |
| `psi(3)` | `9.22784335098467356e-1` | `0.92278433509846747` | `1.110e-16` | yes (1e-12) |

Achieved accuracy is ~1 ulp (≤1.11e-16 absolute), four orders of magnitude
inside the required tolerance. Deltas measured with the identical arithmetic
of `digamma()` in `psi.rs`; the in-repo tests assert the `1e-12` bound and
pass (`observed_scalar_digamma_at_one`,
`observed_vector_digamma_values_and_shape`,
`psi_gpu_gather_matches_observed_values`).

Output class and shape: scalar → `Value::Num` (double 1×1), `[1 2 3]` →
`Value::Tensor` shape `[1, 3]` (double), asserted in the observed tests.

## Traceability (claim → FR → test → result)

All tests live in
`crates/runmat-runtime/src/builtins/math/elementwise/psi.rs`
(`builtins::math::elementwise::psi::tests`). Release applicability: R2026a
(noted in the observed tests' citations).

| Claim / source | FR | Test | Result |
|---|---|---|---|
| psi.signature-primary, psi.output-class, psi.digamma-values | FR-021-01, FR-021-02 | `observed_scalar_digamma_at_one` | PASS |
| psi.digamma-values, psi.output-class (vector case, size 1×3) | FR-021-01..03 | `observed_vector_digamma_values_and_shape` | PASS |
| psi.signature-primary (descriptor surface) | FR-021-01 | `psi_descriptor_signature_covers_single_argument_form` | PASS |
| RunMat type-resolution convention | FR-021-03 | `psi_type_preserves_tensor_shape`, `psi_type_scalar_tensor_returns_num` | PASS |
| psi.q-poles (unresolved) | FR-021-04 | `unresolved_choice_poles_return_nan` | PASS (non-normative) |
| Unobserved negatives / NaN / Inf | FR-021-04 | `unresolved_choice_negative_non_integer_is_finite`, `unresolved_choice_nan_and_infinity_propagation` | PASS (non-normative) |
| Unobserved input promotions | FR-021-04 | `unresolved_choice_int_input_promotes_to_double`, `unresolved_choice_logical_input_promotes_to_double`, `unresolved_choice_char_input_promotes_to_code_points` | PASS (non-normative) |
| Unobserved invalid inputs | FR-021-04 | `unresolved_choice_complex_input_errors`, `unresolved_choice_string_input_errors` | PASS (non-normative) |
| psi.q-polygamma (unresolved) | FR-021-05 | `unresolved_choice_polygamma_form_errors`, `unresolved_choice_extra_arguments_error` | PASS (non-normative) |
| GPU gather-to-host choice | FR-021-04 | `psi_gpu_gather_matches_observed_values` | PASS |
| Algorithm correctness (public mathematics, Principle III/X) | FR-021-02 | `digamma_matches_half_argument_closed_form`, `digamma_satisfies_forward_recurrence` | PASS |

## Deviations and notes

- No `unary_psi` provider hook exists in `runmat-accelerate-api`; the GPU
  spec declares no provider hooks and GPU tensors gather to the host
  (documented in `plan.md`; the accelerate API was intentionally not
  modified — minimal scope). No `accel` macro tag was set, per
  `docs/builtins/authoring.md` (tags only for real device paths).
- The `"like"` output-template option present in `gamma`/`factorial` was
  not carried over: the approved export confirms only `Y = psi(X)`, and any
  second argument must be reserved for the pending polygamma form.
- Registration adds one line to `math/elementwise/mod.rs`; no other existing
  file is modified.
- Licensing (Principle X): algorithm from Abramowitz & Stegun, NBS AMS 55
  (1964), eqs. 6.3.5/6.3.18 — public domain; recorded in `provenance.md`.
  Independent legal review may still be required per the constitution.

## Unresolved (returned to specification side)

`psi.q-polygamma` (two-argument polygamma form `psi(k, X)`),
`psi.q-poles` (behaviour at non-positive integers). Both are implemented by
documented independent choice only and tagged `unresolved_choice_*`.
