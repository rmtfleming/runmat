# Validation Report — 030-accumarray — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (finished, 1m 12s).
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- accumarray` —
  **14 passed, 0 failed, 0 ignored** (6910 filtered out). Full list:
  - `observed_sum_by_subscript_matches_export` ... ok
  - `observed_output_is_double_column` ... ok
  - `unresolved_choice_scalar_vals_broadcast` ... ok
  - `unresolved_choice_sz_extends_output_length` ... ok
  - `unresolved_choice_fillval_used_for_empty_positions` ... ok
  - `unresolved_choice_single_element_output_is_scalar` ... ok
  - `unresolved_choice_multi_column_subs_rejected` ... ok
  - `unresolved_choice_custom_function_handle_rejected` ... ok
  - `unresolved_choice_sparse_output_option_rejected` ... ok
  - `rejects_non_positive_integer_subscripts` ... ok
  - `rejects_non_integer_subscripts` ... ok
  - `rejects_vals_length_mismatch` ... ok
  - `rejects_size_argument_smaller_than_max_subscript` ... ok
  - `accepts_logical_vals_via_coercion` ... ok
- `rustfmt --check` on `array/creation/accumarray.rs` — clean (exit 0).
- `cargo fmt --all -- --check` — the only diffs reported are in **sibling
  Batch-5 files** (`introspection/ver.rs`, `introspection/verLessThan.rs`,
  `math/elementwise/eps.rs`), not in this feature's file; left untouched per
  the concurrent-siblings rule.
- `cargo clippy -p runmat-runtime --lib` — **0 warnings in `accumarray.rs`**
  (build finished clean).

## Traceability (claim → FR → test → result)

| Claim / basis | FR | Test | Result |
|---|---|---|---|
| accumarray.signature-primary, output-class, sum-by-subscript | FR-030-01 | `observed_sum_by_subscript_matches_export`, `observed_output_is_double_column` | PASS |
| documented choice (positive-integer subs) | FR-030-02 | `rejects_non_positive_integer_subscripts`, `rejects_non_integer_subscripts` | PASS |
| documented choice (scalar/length vals) | FR-030-03 | `unresolved_choice_scalar_vals_broadcast`, `rejects_vals_length_mismatch` | PASS |
| documented choice — accumarray.q-size (sz) | FR-030-04 | `unresolved_choice_sz_extends_output_length`, `rejects_size_argument_smaller_than_max_subscript` | PASS |
| documented choice — accumarray.q-size (fillval) | FR-030-05 | `unresolved_choice_fillval_used_for_empty_positions` | PASS |
| unresolved — accumarray.q-func | FR-030-06 | `unresolved_choice_custom_function_handle_rejected` | PASS |
| unresolved — accumarray.q-size (N-D, issparse) | FR-030-07 | `unresolved_choice_multi_column_subs_rejected`, `unresolved_choice_sparse_output_option_rejected` | PASS |
| documented choice (GPU gather / coercion / collapse) | FR-030-08 | `accepts_logical_vals_via_coercion`, `unresolved_choice_single_element_output_is_scalar` | PASS |

## Observed-case fidelity

The single approved observation (`accumarray([1;2;1],[10;20;30])`) reproduces
exactly: value `[40;20]`, class `double` (`Value::Tensor`), size `[2,1]` —
asserted in `observed_sum_by_subscript_matches_export`.

## Deviations and notes

- Placement: `array/creation/accumarray.rs` (constructs an output array from
  subscripts + values, mirroring `sparse`), registered through the
  `#[runtime_builtin]` inventory macro via one `mod` line in
  `array/creation/mod.rs`.
- 1×1 results collapse to `Value::Num`, consistent with `sub2ind` and
  `tensor::tensor_into_value`; length-`>1` results are `[m,1]` tensors.
- Accumulation is a host scatter-add; GPU-resident inputs are gathered first
  (no device kernel). MATLAB was not invoked at any point.

## Unresolved (returned to specification side)

`accumarray.q-func` (custom accumulation function) and `accumarray.q-size`
(size/fill options — the multi-column N-D form and the `issparse` sparse
output remain unimplemented and are rejected with stable identifiers).
