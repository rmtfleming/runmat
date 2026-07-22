# Validation Report — 020-strncmpi — 2026-07-22

## Commands and results (actual runs in this repository)

- `cargo check -p runmat-runtime` — clean
  (`Finished 'dev' profile [unoptimized + debuginfo] target(s) in 1m 19s`)
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::strings::core::strncmpi`
  — **21 passed; 0 failed; 0 ignored** (`test result: ok. 21 passed; 0
  failed; 0 ignored; 0 measured; 6696 filtered out; finished in 0.01s`)
- `cargo fmt` then `cargo fmt --all -- --check` — clean (no diagnostics)

The wgpu-gated test `strncmpi_prefix_length_from_gpu_tensor` compiles under
`--features wgpu` builds only and is not part of the 21 default-feature
tests.

## Traceability (claim → FR → test → result)

| Claim | FR | Test (`builtins::strings::core::strncmpi::tests::…`) | Result |
|---|---|---|---|
| strncmpi.signature-primary, strncmpi.output-class, strncmpi.case-insensitive-prefix | FR-020-01, FR-020-02, FR-020-03 | `observed_match_first_three_characters_ignoring_case_true` (export case `match`: `strncmpi('Hello','help',3)` → true) | PASS |
| strncmpi.output-class, strncmpi.case-insensitive-prefix | FR-020-02, FR-020-03 | `observed_prefix_equal_first_two_characters_true` (export case `prefix-equal`: `strncmpi('abc','abd',2)` → true) | PASS |
| strncmpi.output-class, strncmpi.case-insensitive-prefix | FR-020-02, FR-020-03 | `observed_no_match_first_character_false` (export case `no-match`: `strncmpi('abc','xyz',1)` → false) | PASS |
| strncmpi.q-arrays (unresolved; documented choice) | FR-020-04 | `unresolved_choice_cell_arrays_broadcast_casefold`, `unresolved_choice_string_array_broadcast_scalar_casefold`, `unresolved_choice_char_array_rows_casefold` | PASS (non-normative) |
| (independent choices, unobserved) | FR-020-04 | `unresolved_choice_mismatch_within_prefix_false`, `unresolved_choice_shorter_string_within_prefix_false`, `unresolved_choice_both_exhausted_before_limit_true`, `unresolved_choice_zero_length_always_true`, `unresolved_choice_prefix_length_bool_true_compares_first_character`, `unresolved_choice_prefix_length_bool_false_treated_as_zero`, `unresolved_choice_prefix_length_logical_array_scalar`, `unresolved_choice_prefix_length_tensor_scalar_double`, `unresolved_choice_missing_string_false_when_prefix_positive`, `unresolved_choice_missing_zero_length_true`, `unresolved_choice_size_mismatch_error`, `unresolved_choice_invalid_length_type_errors`, `unresolved_choice_negative_length_errors`, `unresolved_choice_invalid_argument_type_errors` | PASS (non-normative) |
| (infrastructure) | SC-020-2 | `strncmpi_type_is_logical_match`; registration via `#[runtime_builtin]` inventory macro exercised by the module test run | PASS |

All 3 observed cases in the `strncmpi` 0.2.0 export reproduce exactly
(value, class logical, size 1×1 — `Value::Bool` is RunMat's 1×1 logical).
SC-020-1, SC-020-2 and SC-020-3 are satisfied.

## Deviations and notes

- No deviations from the approved export. The observed tier is implemented
  and asserted exactly.
- Array/length/missing/error semantics beyond the three observed cases are
  independent choices mirroring RunMat's existing `strncmp` (structure and
  broadcasting) and `strcmpi` (Unicode lowercase fold), tagged
  `unresolved_choice_*` and never asserted as MATLAB-conformant.
- `strings/core/mod.rs` gained one line (`pub mod strncmpi;`); a concurrent
  sibling feature added `pub mod dec2bin;` to the same file — both entries
  verified intact after the edit.
- Constitution X note: no external implementation sources were used;
  independent legal review may still be required per programme policy.

## Unresolved (returned to specification side)

`strncmpi.q-arrays` — behaviour with cell-array inputs (element-wise);
additionally flagged for observation: length-versus-`n` handling (strings
shorter than `n`), `n = 0`, missing string elements, and invalid-argument
error identities.
