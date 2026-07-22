# Validation Report — 007-numeric-string-conversion — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (first run 1m 38s cold, 15s warm)
- `cargo fmt --all -- --check` — clean
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::strings::core::int2str builtins::strings::core::str2num` — **27 passed, 0 failed** (12 `int2str`, 15 `str2num`)

## Traceability (claim → FR → test → result)

| Claim | FR | Test | Result |
|---|---|---|---|
| int2str.signature-primary, int2str.output-class | FR-007-01 | `int2str::tests::observed_scalar_rounds_to_nearest_integer` (char class/shape asserted in every observed test) | PASS |
| int2str.rounding-and-char | FR-007-02 | `int2str::tests::observed_scalar_rounds_to_nearest_integer`, `observed_negative_half_rounds_away_from_zero` | PASS |
| int2str.rounding-and-char (multi-element layout, observed extents) | FR-007-03 | `int2str::tests::observed_vector_elements_joined_with_two_spaces` (1×7), `observed_matrix_rows_mirror_input_rows` (2×4) | PASS |
| str2num.signature-primary, str2num.output-class | FR-007-04 | `str2num::tests::observed_scalar_literal` (double class/shape asserted in every observed test) | PASS |
| str2num.parse-eval | FR-007-05 | `str2num::tests::observed_scalar_literal`, `observed_bracketed_row_vector`, `observed_bracketed_matrix`, `observed_arithmetic_expression`; precedence: `summary_derived_arithmetic_precedence_and_grouping` | PASS |
| (summary) unparsable→empty | FR-007-06 | `str2num::tests::summary_derived_unparsable_text_returns_empty` | PASS (summary-derived) |
| unresolved choices | FR-007-07 | `int2str::…::unresolved_choice_columns_right_aligned_two_space_separator`, `unresolved_choice_nonfinite_values_keep_word_spellings`, `unresolved_choice_empty_input_returns_empty_char`, `unresolved_choice_integer_and_logical_inputs_format_numerically`, `unresolved_choice_non_numeric_input_errors`; `str2num::…::unresolved_choice_comma_separated_row`, `unresolved_choice_signed_whitespace_tokenisation`, `unresolved_choice_empty_brackets_and_bracketed_scalar`, `unresolved_choice_inconsistent_row_lengths_return_empty`, `unresolved_choice_identifier_expression_returns_empty_without_vm`, `unresolved_choice_non_text_input_errors`, `unresolved_choice_string_scalar_input_accepted` | PASS (non-normative) |
| Infrastructure (Principle IX; SC-007-2) | — | `int2str::tests::int2str_gpu_tensor_roundtrip`, `int2str_is_registered`, `int2str_type_is_string_scalar`, `str2num::tests::str2num_is_registered`, `str2num_type_resolver_is_unknown` | PASS |

All 8 observed cases across the two exports reproduce exactly (value, class,
size — char via `Value::CharArray` rows/cols, double via `Value::Num` ≡ 1×1
and `Value::Tensor` shape/column-major data, per RunMat's value model).

## Deviations and notes

- `str2num` delegation: the task's suggested
  `call_builtin_async("eval", …)` path is implemented, but RunMat's
  runtime-side `eval` registration is a stub that requires a VM workspace
  frame (`RunMat:DynamicWorkspaceRequiresVm`); the VM intercepts `eval`
  only at its own call sites (`runmat-vm/src/call/builtins.rs`), so
  runtime-internal delegation always fails today — including under the VM.
  The observed grammar (literals, bracketed vectors/matrices, arithmetic
  expressions) is therefore covered by a self-contained clean-room
  parser/evaluator; out-of-grammar text goes through the delegation path
  and maps to the documented empty result. Revisit when/if a runtime↔VM
  eval bridge exists (tracked under `str2num.q-eval`).
- Rounding uses Rust's `f64::round` (ties away from zero), which
  reproduces the observed ties; no bespoke rounding algorithm was written.
- WASM builtin registry (`generated_wasm_registry.rs`) is untouched: native
  builds skip its validation; it must be regenerated via
  `scripts/regenerate-wasm-registry.sh` before the next wasm build.
- Registration is via the `#[runtime_builtin]` inventory macro
  (compile-time, same mechanism as existing builtins).

## Unresolved (returned to specification side)

`int2str.q-rounding`, `int2str.q-spacing`, `str2num.q-eval`,
`str2num.q-second-output`.
