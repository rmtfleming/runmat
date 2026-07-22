# Validation Report — 018-dec2bin — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (Finished in 16.29s)
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::strings::core::dec2bin`
  — **16 passed, 0 failed, 0 ignored** (6689 filtered out)
- `rustfmt --edition 2021 --check` on `dec2bin.rs` and `strings/core/mod.rs`
  — clean (exit 0). `cargo fmt --all -- --check` reports diffs only in a
  concurrent sibling's file (`math/sparse/nonzeros.rs`), not in any file this
  feature owns.

## Observed cases reproduced exactly (normative)

| export case | call | expected | test | result |
|---|---|---|---|---|
| `scalar` | `dec2bin(5)` | `'101'` (char, 1×3) | `observed_scalar_five_is_101` | PASS |
| `padded` | `dec2bin(5, 8)` | `'00000101'` (char, 1×8) | `observed_padded_width_eight_zero_pads` | PASS |
| `zero` | `dec2bin(0)` | `'0'` (char, 1×1) | `observed_zero_is_single_zero_digit` | PASS |
| `width-below-natural` | `dec2bin(5, 2)` | `'101'` (char, 1×3) | `observed_width_below_natural_does_not_truncate` | PASS |

## Traceability (claim → FR → test → result)

| Claim | FR | Test | Result |
|---|---|---|---|
| dec2bin.signature-primary, output-class, binary-char | FR-018-01, FR-018-02, FR-018-03 | `observed_scalar_five_is_101`, `observed_zero_is_single_zero_digit` | PASS |
| dec2bin.binary-char, output-class | FR-018-04 | `observed_padded_width_eight_zero_pads` | PASS |
| dec2bin.binary-char | FR-018-05 | `observed_width_below_natural_does_not_truncate` | PASS |
| unresolved (dec2bin.q-vector) | FR-018-06 | `unresolved_choice_vector_rows_share_common_width`, `unresolved_choice_vector_with_width_pads_every_row`, `unresolved_choice_empty_input_returns_empty_char` | PASS (non-normative) |
| unresolved (dec2bin.q-negative) | FR-018-06 | `unresolved_choice_negative_input_errors`, `unresolved_choice_noninteger_rounds_to_nearest`, `unresolved_choice_nonfinite_input_errors` | PASS (non-normative) |
| unresolved (width/class choices) | FR-018-06 | `unresolved_choice_invalid_width_arguments_error`, `unresolved_choice_integer_and_logical_inputs_convert`, `unresolved_choice_non_numeric_input_errors` | PASS (non-normative) |
| registration/type/GPU (SC-018-2, Principle IX) | — | `dec2bin_is_registered`, `dec2bin_type_is_string_scalar`, `dec2bin_gpu_tensor_roundtrip` | PASS |

All four observed cases reproduce exactly (value, class char, size). Every FR
maps to ≥1 test; every normative test maps to ≥1 FR and claim id (SC-018-3).

## Actual outputs (from the test run)

- `dec2bin(5)` → `'101'`, rows=1, cols=3
- `dec2bin(5, 8)` → `'00000101'`, rows=1, cols=8
- `dec2bin(0)` → `'0'`, rows=1, cols=1
- `dec2bin(5, 2)` → `'101'`, rows=1, cols=3
- `dec2bin([5 3])` → rows `'101'`,`'011'`, 2×3 (documented choice)
- `dec2bin(-1)` → error `RunMat:dec2bin:InvalidInput` (documented choice)

## Deviations and notes

- Multi-element inputs and negative/non-integer handling are unobserved in
  the approved export; both are implemented by documented independent choice
  (spec Unresolved behaviour): one row per element (column-major) zero-padded
  to a common width; non-integers round to nearest (ties away from zero);
  negatives, non-finite values and values above 2^53 error.
- The optional width argument is parsed like `num2str`'s trailing options
  (`rest: Vec<Value>`); a non-scalar/non-numeric/negative/non-finite width, or
  more than two arguments, errors with `RunMat:dec2bin:InvalidWidth`.
- Registration is via the `#[runtime_builtin]` inventory macro (compile-time,
  the same mechanism as all existing builtins); discoverability is exercised
  by `dec2bin_is_registered`.
- No file owned by any other concurrent feature was modified; the only shared
  edit is one added `pub mod dec2bin;` line in `strings/core/mod.rs`.

## Unresolved (returned to specification side)

`dec2bin.q-vector` (vector/matrix char-matrix layout), `dec2bin.q-negative`
(negative and non-integer input behaviour).
