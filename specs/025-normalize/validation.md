# Validation Report — 025-normalize — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (exit 0). (A transient error in
  sibling batch files `math/sparse/colamd.rs` / `dissect.rs` — `matrix_pattern`
  import, not part of this feature — appeared during a concurrent sibling edit
  and cleared on re-run; `normalize.rs` never produced an error.)
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::reduction::normalize`
  — **17 passed, 0 failed, 0 ignored** (6845 filtered out).
- `cargo fmt -p runmat-runtime` then `cargo fmt --all -- --check` — clean
  (exit 0).

## Observed cases reproduced exactly

| Case (observation) | Call | Expected | Test | Result |
|---|---|---|---|---|
| `zscore` | `normalize([1 2 3])` | `[-1 0 1]`, double, 1×3 | `observed_zscore_vector`, `observed_output_class_is_double_tensor` | PASS |
| `range` | `normalize([1 2 3],'range')` | `[0 0.5 1]`, double, 1×3 | `observed_range_vector` | PASS |

Sample-std (N−1) confirmed: variance `((1-2)²+(0)+(1)²)/(3-1) = 2/2 = 1`, so
`std = 1` and `(x-2)/1 = [-1 0 1]`, matching the observation exactly.

## Traceability (claim / question → FR → test → result)

| Claim / question | FR | Test | Result |
|---|---|---|---|
| normalize.signature-primary, normalize.output-class | FR-025-01, FR-025-07 | `observed_zscore_vector`, `observed_output_class_is_double_tensor`, `integer_input_produces_double` | PASS |
| normalize.zscore-and-range (z-score) | FR-025-02 | `observed_zscore_vector` | PASS |
| normalize.zscore-and-range (range) | FR-025-03 | `observed_range_vector` | PASS |
| normalize.q-dim (documented) | FR-025-04 | `unresolved_choice_matrix_default_is_columnwise`, `unresolved_choice_explicit_dim_rowwise` | PASS (non-normative) |
| normalize.q-methods (documented) | FR-025-05 | `unresolved_choice_explicit_zscore_matches_default`, `unresolved_choice_range_custom_interval`, `unresolved_choice_center_subtracts_mean`, `unresolved_choice_scale_divides_by_std`, `unresolved_choice_norm_default_two`, `unresolved_choice_norm_order_one` | PASS (non-normative) |
| normalize.q-methods (rejected/unresolved) | FR-025-06 | `unsupported_medianiqr_is_rejected`, `unsupported_zscore_robust_is_rejected`, `complex_input_is_unsupported`, `unknown_method_is_invalid_argument`, `descriptor_has_stable_error_codes` | PASS |

All 17 tests: 2 observed/normative reproductions (+1 class check, +1
integer-input check), 6 documented-choice (`unresolved_choice_*`), 5
error/rejection, 1 descriptor invariant.

## Deviations and notes

- Registration is via the compile-time `#[runtime_builtin]` inventory macro
  plus `pub(crate) mod normalize;` in `math/reduction/mod.rs` — the same
  mechanism as all sibling reduction builtins. GPU and fusion specs are
  registered (`register_gpu_spec` / `register_fusion_spec`) as a
  gather-to-host composite (`GpuOpKind::Custom("reduction")`,
  `ResidencyPolicy::GatherImmediately`).
- Output class: the result is always a real double `Value`
  (`tensor::tensor_into_value`; scalar collapses to `Value::Num`), honouring
  the observed `double` class regardless of input numeric class.
- Documented independent choices (unobserved; NOT asserted MATLAB-conformant):
  first-non-singleton default dimension; explicit `dim`; `center`/`scale`/
  `norm`/custom `range`; NaN propagation for z-score/center/scale and NaN
  omission for range min/max; degenerate scale (std 0, single element) → NaN.
- Clean-room: no MATLAB invoked; no MathWorks source/docs/examples used. The
  sample-std interpretation is derived from the standard definition and the
  observed result, not copied.

## Unresolved (returned to specification side)

`normalize.q-dim` (matrix operating dimension), `normalize.q-methods` (norm,
center, scale, medianiqr and robust variants — the robust/median/MAD/IQR
family is deliberately unimplemented and rejected with
`RM.NORMALIZE.UNSUPPORTED` pending spec-side confirmation).
