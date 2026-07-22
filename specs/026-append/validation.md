# Validation Report — 026-append — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (sibling Tier B batch-2 features
  `colamd`/`dissect` briefly broke the shared crate mid-edit; re-run after they
  compiled was clean, per the concurrent-build protocol — `append.rs` was never
  the cause).
- `cargo fmt` then `cargo fmt --all -- --check` — clean (exit 0).
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::strings::transform::append`
  — **12 passed, 0 failed, 0 ignored** (6850 filtered out).

### Actual observed-case outputs (from the passing normative tests)

| call | result value | class | size |
|---|---|---|---|
| `append("a","b")` | `"ab"` (`Value::String`) | string | `[1, 1]` |
| `append("x","y","z")` | `"xyz"` (`Value::String`) | string | `[1, 1]` |
| `append('a','b')` | `'ab'` (`Value::CharArray`, data `['a','b']`) | char | `[1, 2]` |

All three reproduce the export exactly.

## Traceability (claim → FR → test → result)

| Claim / basis | FR | Test | Result |
|---|---|---|---|
| append.signature-primary, append.concatenates-text | FR-026-01, FR-026-03 | `append::tests::observed_two_strings_return_string_scalar` | PASS |
| append.concatenates-text | FR-026-02, FR-026-03 | `observed_three_strings_return_string_scalar` | PASS |
| append.concatenates-text | FR-026-02, FR-026-03 | `observed_two_chars_return_char_row` | PASS |
| documented choice (no trim) | FR-026-04 | `unresolved_choice_char_inputs_preserve_trailing_whitespace` | PASS (non-normative) |
| documented choice (output class) | FR-026-02, FR-026-05 | `unresolved_choice_mixed_char_and_string_returns_string` | PASS (non-normative) |
| documented choice (single arg) | FR-026-05 | `unresolved_choice_single_string_argument_returns_itself` | PASS (non-normative) |
| documented choice (string array) | FR-026-05 | `unresolved_choice_string_array_concatenates_element_wise` | PASS (non-normative) |
| documented choice (empty array) | FR-026-05 | `unresolved_choice_empty_string_array_returns_empty_array` | PASS (non-normative) |
| documented choice (zero args) | FR-026-05 | `unresolved_choice_no_arguments_errors` | PASS (non-normative) |
| documented choice (non-text) | FR-026-05 | `unresolved_choice_non_text_input_errors` | PASS (non-normative) |
| documented choice (size mismatch) | FR-026-05 | `unresolved_choice_mismatched_sizes_error` | PASS (non-normative) |
| type inference | FR-026-02 | `append_type_resolver_reports_string_for_string_input` | PASS |
| GPU-operand type safety (wgpu only) | FR-026-05 | `append_gpu_operand_still_errors_on_type` | PASS under `--features wgpu` (not run in default profile) |

Every FR maps to ≥1 test; every observed case maps to a normative test; no
orphan normative tests.

## Deviations and notes

- **Scope choice — cellstr excluded.** `strcat` (the exemplar) accepts cell
  arrays of char; `append` here rejects them with `RunMat:append:InvalidInput`.
  Cell-array behaviour is unobserved for `append`; excluding it keeps scope
  minimal (Principle VII). If required it must return to the specification side.
- **Key contrast with `strcat`.** `append` preserves trailing whitespace on
  char inputs; `strcat` trims it. Unobserved for `append`; implemented as a
  documented independent choice (FR-026-04) and pinned by
  `unresolved_choice_char_inputs_preserve_trailing_whitespace`.
- **Missing-string sentinels** are not special-cased (plain concatenation),
  unlike `strcat`'s `<missing>` propagation. Unobserved; documented choice.
- Registration is via the compile-time `#[runtime_builtin]` inventory macro —
  the same mechanism used by every existing builtin — so no explicit registry
  edit is needed beyond the `mod append;` line.

## Unresolved (returned to specification side)

`append.q-release` (introducing release; implemented unconditionally),
`append.q-types` (full char/string/mixed output-type matrix; only the observed
subset asserted normative). Plus the documented choices above, which have no
approved backing and are candidates for future observation: trailing-whitespace
preservation, string-array element-wise semantics, empty/single-argument
handling, and non-text/cellstr input handling.
