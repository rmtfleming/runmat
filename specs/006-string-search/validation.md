# Validation Report — 006-string-search — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (18s)
- `cargo fmt --all -- --check` — clean
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::strings::search::matches builtins::strings::search::strmatch builtins::strings::search::strtok` — **32 passed, 0 failed**

## Traceability (claim → FR → test → result)

| Claim | FR | Test | Result |
|---|---|---|---|
| matches.signature-primary, output-class, scalar-membership | FR-006-01 | `matches::tests::observed_in_set_membership_is_true`, `observed_scalar_char_equality_is_true` | PASS |
| matches.output-class, scalar-membership (IgnoreCase) | FR-006-02 | `matches::tests::observed_ignore_case_option_is_true` | PASS |
| unresolved choices (matches) | FR-006-03 | `matches::tests::unresolved_choice_non_member_is_false`, `unresolved_choice_case_sensitive_default_is_false`, `unresolved_choice_array_first_argument_shapes_result`, `unresolved_choice_cell_patterns_membership`, `unresolved_choice_missing_subject_is_false` | PASS (non-normative) |
| strmatch.signature-primary, output-class, index-vector | FR-006-04 | `strmatch::tests::observed_prefix_match_indices`, `observed_no_match_is_zero_by_one_empty` | PASS |
| strmatch.output-class, index-vector ('exact') | FR-006-05 | `strmatch::tests::observed_exact_match_index` | PASS |
| unresolved choices (strmatch) | FR-006-06 | `strmatch::tests::unresolved_choice_char_matrix_rows`, `unresolved_choice_string_array_entries`, `unresolved_choice_exact_excludes_prefix_only` | PASS (non-normative) |
| strtok.signature-primary, output-class, leading-token | FR-006-07 | `strtok::tests::observed_default_whitespace_token`, `observed_leading_delimiters_skipped` | PASS |
| strtok.output-class, leading-token (delimiter set) | FR-006-08 | `strtok::tests::observed_custom_delimiter_token` | PASS |
| (summary) second output returns the remainder | FR-006-09 | `strtok::tests::summary_derived_two_output_remainder`, `summary_derived_custom_delimiter_remainder` | PASS (summary-derived) |
| unresolved choices (strtok) | FR-006-10 | `strtok::tests::unresolved_choice_empty_and_all_delimiter_inputs`, `unresolved_choice_string_input_returns_string`, `unresolved_choice_other_whitespace_delimits` | PASS (non-normative) |

Error-path coverage (RunMat-specific diagnostics, non-normative):
`matches::tests::{invalid_option_name_errors, invalid_subject_type_errors,
invalid_pattern_type_errors}`,
`strmatch::tests::{invalid_flag_errors, invalid_pattern_type_errors,
invalid_array_type_errors}`,
`strtok::tests::{invalid_input_errors, invalid_delimiter_errors}` — all PASS.
Type-resolver checks: `matches::tests::matches_type_is_logical_match`,
`strmatch::tests::strmatch_type_is_tensor` — PASS.

All 9 observed cases across the three exports reproduce exactly (value,
class, size: logical 1×1 via `Value::Bool`; double 2×1/1×1/0×1 via
`Value::Tensor`; char 1×5/1×1/1×4 via `Value::CharArray`, per RunMat's value
model).

## Deviations and notes

- `matches` result shape: the export observes scalar first arguments only.
  The implementation shapes the result like the first argument and tests
  each element for equality against the whole pattern set — a documented
  independent choice consistent with the observed membership rule; it is
  not the broadcast semantics used by `contains`, because broadcast would
  contradict the observed `in-set` case (1×2 pattern → 1×1 result).
- `strmatch` emits no legacy/deprecation warning (`strmatch.q-deprecated`
  unresolved); flags other than `'exact'` are rejected with
  `RunMat:strmatch:InvalidFlag`.
- `strtok` remainder starts at (and includes) the delimiter that terminated
  the token; empty/all-delimiter input yields empty token (0×0 char) and
  empty remainder — documented choices under `strtok.q-remainder` /
  `strtok.q-empty`.
- Registration is via the `#[runtime_builtin]` inventory macro
  (compile-time, same mechanism as the existing builtins); `strtok`
  multi-output uses the `output_count` machinery shared with `fileparts`.

## Unresolved (returned to specification side)

`matches.q-release`, `matches.q-string-vs-char`, `strmatch.q-deprecated`,
`strmatch.q-shape` (n-D orientation), `strtok.q-remainder` (observation for
the second output), `strtok.q-empty`.
