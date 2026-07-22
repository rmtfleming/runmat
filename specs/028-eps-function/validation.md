# Validation Report — 028-eps-function — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean.
- `cargo fmt --all -- --check` — clean (exit 0).
- `cargo clippy -p runmat-runtime --lib` — clean (exit 0); no warnings in
  `eps.rs` (the only file authored for this feature).
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::elementwise::eps`
  — **18 passed, 0 failed**.
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- eps` (swap-safety
  filter; catches any test referencing `eps` across the crate) — **56 passed,
  0 failed**. No existing test broke from the constant → function swap. (Repo
  grep confirmed no test asserted the `eps` constant registration, and no
  snapshot/integration test referenced the MATLAB `eps` name; the other `eps`
  matches are unrelated local variables `eps`/`EPS` and the `"eps"` print
  device token.)

## Bare-`eps` resolution evidence (key risk)

Removing the `eps` constant made bare `eps` depend on the VM resolving it as a
zero-argument builtin call. Verified two ways:

1. **Registry level** (`eps_is_registered_as_builtin_and_not_constant`, PASS):
   `builtin_function_by_name("eps").is_some()` is true and `constants()`
   contains no `eps`, while `pi/inf/Inf/nan/NaN/i/j` remain. These are exactly
   the two registries the HIR `lower_expr_semantic` bare-identifier arm
   consults (`runmat_builtins::constants()` then `is_builtin` →
   `builtin_functions()`).
2. **End-to-end** through parser → HIR → VM (temporary `runmat-core`
   integration test, run then removed):

   | Source | Result |
   |---|---|
   | `d = eps` | `Num(2.2204460492503131e-16)` (= `f64::EPSILON`) |
   | `eps(2)` | `Num(2^-51)` (documented ulp) |
   | `class(eps('single'))` | `"single"` |
   | `eps('single')` | `Tensor { data:[1.1920928955078125e-7], shape:[1,1], dtype:F32 }` |

   All assertions passed (`test result: ok. 1 passed`).

**Outcome: bare `eps` resolves correctly as a zero-argument builtin call — not
a blocker.**

## Actual observed outputs (exact)

| case | call | RunMat result | class | size |
|---|---|---|---|---|
| `default` | `eps` | `2.2204460492503131e-16` | double | 1×1 |
| `single-class` | `eps('single')` | `1.1920928955078125e-07` | single | 1×1 |

Both match the approved observations byte-for-byte
(`f64::EPSILON` and `f32::EPSILON as f64`).

## Traceability (claim / choice → FR → test → result)

| Source | FR | Test | Result |
|---|---|---|---|
| eps.signature-primary, eps.spacing-constants | FR-028-01 | `observed_bare_eps_is_double_epsilon` | PASS (normative) |
| eps.spacing-constants | FR-028-02 | `observed_eps_single_is_f32_epsilon` | PASS (normative) |
| documented choice (eps.q-value-form) | FR-028-03 | `unresolved_choice_eps_of_one_equals_epsilon`, `…_of_two_is_ulp`, `…_negative_uses_magnitude`, `…_of_zero_is_smallest_subnormal`, `…_of_inf_and_nan_are_nan`, `…_double_array_preserves_shape`, `…_single_array_is_single`, `…_complex_uses_magnitude`, `…_complex_tensor_is_real` | PASS (non-normative) |
| documented choice | FR-028-04 | `unresolved_choice_eps_double_classname`, `…_single_classname_case_insensitive`, `…_invalid_class_errors` | PASS (non-normative) |
| documented choice | FR-028-05 | `unresolved_choice_eps_too_many_args_errors`, `…_integer_input_errors` | PASS (non-normative) |
| constant→function migration | FR-028-06 | `eps_is_registered_as_builtin_and_not_constant` + end-to-end above | PASS |
| descriptor/macro | SC-028-3 | `eps_descriptor_signatures_cover_core_forms` | PASS |

Every normative test cites an approved claim; every documented-choice test is
named `unresolved_choice_*` and carries the `eps.q-value-form` question.

## Deviations and notes

- **Constant → function migration.** Only the single
  `register_constant!("eps", Value::Num(f64::EPSILON), …)` entry was removed
  from `constants/mod.rs` (an explanatory comment replaces it); `pi`, `sqrt2`,
  `i`, `j`, `inf`/`Inf`, `nan`/`NaN`, `true`, `false` are untouched.
- **Single scalar representation.** `eps('single')` returns a 1×1
  `NumericDType::F32` tensor (not `Value::Num`, and not collapsed to `Num`) so
  `class(...)` reports `single`. This mirrors how `ones(1,1,'single')` etc.
  represent single scalars in RunMat.
- **`eps(x)` algorithm.** Standard IEEE `nextUp(|x|) − |x|` spacing (public
  numerical technique), computed in the input precision. This is a documented
  independent choice for the unobserved value form, not an inferred MATLAB
  internal. Edge behaviour follows `nextUp` (`eps(0)` = smallest subnormal,
  `eps(Inf)`/`eps(NaN)` = `NaN`, `eps(realmax)` = `Inf`).
- **WASM builtin registry.** Adding `eps.rs` changes the source fingerprint, so
  `generated_wasm_registry.rs` is stale for **wasm** builds until
  `scripts/regenerate-wasm-registry.sh` is run. Native `cargo check`/tests are
  unaffected (`build.rs` skips registry validation when
  `target_arch != wasm32`). Regeneration is deferred to the batch level because
  five sibling features add builtins concurrently; regenerating now would be
  immediately stale. Follow-up, not a native blocker.
- **`docs/builtins/index.md`** still lists `eps` in the `constants` row
  (line ~182). Not updated here (out of the requested deliverables, and no
  automated doc-coverage test consumes it); flagged as a follow-up doc edit.
- No MATLAB was invoked. Independent legal review may still be required per
  Constitution Principle X.

## Unresolved (returned to specification side)

`eps.q-value-form` — the `eps(x)` value-dependent form (and, by extension,
`eps('double')`, array/negative/complex inputs, and the subnormal/Inf/NaN
edges) remains unobserved in the approved export and is implemented by
documented choice only.
