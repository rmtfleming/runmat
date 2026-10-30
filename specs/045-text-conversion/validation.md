# Validation Report — 045-text-conversion — 2026-10-30

Environment:
- `HDF5_DIR=/home/rfleming/anaconda3`.
- Cargo runner wrapper (`run.sh`, which sets `LD_LIBRARY_PATH` per process
  only).
- Rust 1.90.0.
- The target directory was shared with three parallel N1 agents.
- MATLAB was not invoked.

## Freshness

The check ran in `../matlab-interface-spec`, read-only, both at the start
and at the finish (2026-10-30T01:28Z):

- `git fetch -q --all` exited 0.
- HEAD = origin/main = `8f72edb`.
- `git diff --name-only c65e238 {HEAD,origin/main} -- specifications/{regexptranslate,dec2base,validatestring}` returned nothing.
- `git status --short` was clean.

Result: the `c65e238` import is current.

## Claim-id check

Every `regexptranslate.*`, `dec2base.*` and `validatestring.*` claim id
cited in `spec.md` and `provenance.md` was grepped against the imported
`provenance.yaml`: 27 present, 0 missing. All cited `*.q-*` ids exist under
`unresolved_questions` in the imported `interface.yaml`.

## Test-first evidence (T6)

Setup:
- The three files contained only their test modules.
- `pub mod` lines were registered in the hand-written `strings/regex/mod.rs`
  and `strings/core/mod.rs`.

Command:

`cargo test -p runmat-runtime --lib -- regexptranslate_tests dec2base_tests validatestring_tests isprop_tests`

Result: **0 passed, 74 failed**. Every failure was `RunMat:UndefinedFunction`:

| Feature | Failed tests | "Undefined function" occurrences |
|---|---|---|
| regexptranslate | 18 | 15 |
| dec2base | 23 | 15 |
| validatestring | 17 | 8 |

Some tests assert through helpers before reaching a call that names the
function, which is why the occurrence counts are lower than the test
counts. The isprop tests in the same run are reported in
`specs/046-isprop/validation.md`.

## Commands and results (T10)

- **Feature tests.** `cargo test -p runmat-runtime --lib -- regexptranslate_tests dec2base_tests validatestring_tests isprop_tests`
  gave **74 passed, 0 failed**. Of those, 58 belong to this feature: 18
  regexptranslate, 23 dec2base and 17 validatestring.
  - regexptranslate, observed (13): `observed_escape_dot`,
    `observed_escape_specials`, `observed_escape_plain`,
    `observed_escape_others`, `observed_escape_whitespace`,
    `observed_wildcard_star`, `observed_wildcard_question`,
    `observed_wildcard_dot`, `observed_wildcard_use`,
    `observed_string_input`, `observed_cell_input`, `observed_empty`,
    `observed_unknown_operation_errors`.
  - regexptranslate, other (5): `documented_choice_arity_and_op_forms`,
    `unresolved_case_variant_ops_are_unsupported`,
    `documented_choice_unlisted_characters_are_copied`,
    `documented_choice_shapes_and_unsupported_inputs`,
    `unresolved_no_warning_emitted`.
  - dec2base, observed (17): `observed_base2`, `observed_base3`,
    `observed_base16`, `observed_base36_pair`, `observed_zero`,
    `observed_base36_z`, `observed_pad`, `observed_pad_short`,
    `observed_column_vector`, `observed_row_vector`,
    `observed_two_char_symbol_base_errors`, `observed_uint8`,
    `observed_two_pow_52`, `observed_negative_errors`,
    `observed_fraction_errors`, `observed_base1_errors`,
    `observed_base37_errors`.
  - dec2base, other (6): `documented_choice_shapes_and_classes_of_d`,
    `documented_choice_base_forms`, `unresolved_char_bases_are_unsupported`,
    `unresolved_values_above_two_pow_52_are_unsupported`,
    `documented_choice_digit_count_forms`, `unresolved_no_warning_emitted`.
  - validatestring, observed (13): `observed_exact`, `observed_prefix`,
    `observed_case_insensitive`, `observed_cell_of_strings_errors`,
    `observed_string_input_with_cellstr_list`, `observed_string_list`,
    `observed_exact_over_prefix`, `observed_prefix_of_nested_options`,
    `observed_ambiguous_errors`, `observed_no_match_errors`,
    `observed_funcname_keeps_runmat_identifier`,
    `observed_numeric_input_errors`, `observed_empty_input_errors`.
  - validatestring, other (4): `documented_choice_arity_and_names`,
    `unresolved_more_than_four_arguments_are_unsupported`,
    `documented_choice_lists_text_and_ties`,
    `unresolved_no_warning_emitted`.
- **Strings regressions.** `cargo test -p runmat-runtime --lib -- builtins::strings`
  gave **1056 passed, 0 failed**, including the 58 new tests.
- **Formatting.**
  `rustfmt --edition 2021 --check` on the three files is clean.
- **Clippy.**
  `cargo clippy -p runmat-runtime --lib --tests --message-format short`
  reports **no diagnostics in any of the three files**. The overall run
  still exits non-zero, because all 16 errors are in other parallel agents'
  new files: `graph/domain/editing.rs` (3), `math/linalg/factor/balance.rs`
  (11), `math/sparse/amd.rs` (1) and `math/sparse/etree.rs` (1).
- **Release strings.** No release string appears in the code files.

## Traceability (claim → FR → test → result)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| regexptranslate.signature-primary, .escape | FR-045-01, 02 | `observed_escape_dot`, `observed_escape_specials`, `observed_escape_plain`, `observed_escape_others` | PASS |
| regexptranslate.escape-whitespace | FR-045-02 | `observed_escape_whitespace` | PASS |
| regexptranslate.wildcard | FR-045-03 | `observed_wildcard_star`, `observed_wildcard_question`, `observed_wildcard_dot` | PASS |
| regexptranslate.wildcard-use | FR-045-03 | `observed_wildcard_use` | PASS |
| regexptranslate.types | FR-045-04 | `observed_string_input`, `observed_cell_input`, `observed_empty` | PASS |
| regexptranslate.error-op | FR-045-05 | `observed_unknown_operation_errors` | PASS |
| dec2base.signature-primary, .output-class, .digits | FR-045-06, 07 | `observed_base2`, `observed_base3`, `observed_base16`, `observed_base36_z` | PASS |
| dec2base.zero | FR-045-07 | `observed_zero` | PASS |
| dec2base.padding | FR-045-08 | `observed_pad`, `observed_pad_short` | PASS |
| dec2base.vector | FR-045-09 | `observed_column_vector`, `observed_row_vector`, `observed_base36_pair` | PASS |
| dec2base.integer-class, .large | FR-045-10 | `observed_uint8`, `observed_two_pow_52` | PASS |
| dec2base.error-negative | FR-045-11 | `observed_negative_errors`, `observed_fraction_errors` | PASS |
| dec2base.error-base1 | FR-045-11 | `observed_base1_errors`, `observed_base37_errors`, `observed_two_char_symbol_base_errors` | PASS |
| validatestring.signature-primary, .exact | FR-045-12, 13 | `observed_exact` | PASS |
| validatestring.prefix, .case-insensitive | FR-045-13 | `observed_prefix`, `observed_case_insensitive` | PASS |
| validatestring.nested-options | FR-045-14 | `observed_exact_over_prefix`, `observed_prefix_of_nested_options` | PASS |
| validatestring.error-ambiguous | FR-045-14 | `observed_ambiguous_errors` | PASS |
| validatestring.output-type | FR-045-15 | `observed_exact`, `observed_string_input_with_cellstr_list`, `observed_string_list` | PASS |
| validatestring.error-nomatch | FR-045-16 | `observed_no_match_errors`, `observed_empty_input_errors`, `observed_numeric_input_errors` | PASS |
| validatestring.error-funcname | FR-045-12, 16 | `observed_funcname_keeps_runmat_identifier` | PASS |
| validatestring.error-cell-of-strings | FR-045-17 | `observed_cell_of_strings_errors` | PASS |
| documented choice: arity and op forms | FR-045-18 | `documented_choice_arity_and_op_forms` | PASS |
| unresolved: regexptranslate.q-other-ops | FR-045-18 | `unresolved_case_variant_ops_are_unsupported` | PASS |
| documented choice: unlisted characters | FR-045-19 | `documented_choice_unlisted_characters_are_copied` | PASS |
| documented choice: shapes, unsupported subjects | FR-045-20 | `documented_choice_shapes_and_unsupported_inputs` | PASS |
| documented choice: d classes, shapes, NaN/Inf, arity | FR-045-21 | `documented_choice_shapes_and_classes_of_d` | PASS |
| documented choice: base forms | FR-045-22 | `documented_choice_base_forms` | PASS |
| unresolved: dec2base.q-char-base (D-045-2) | FR-045-22 | `unresolved_char_bases_are_unsupported`, `observed_two_char_symbol_base_errors` | PASS |
| unresolved: dec2base.q-large-values | FR-045-23 | `unresolved_values_above_two_pow_52_are_unsupported` | PASS |
| documented choice: n forms | FR-045-24 | `documented_choice_digit_count_forms` | PASS |
| documented choice: arity, 3-arg form, names | FR-045-25 | `documented_choice_arity_and_names` | PASS |
| unresolved: validatestring.q-more-args | FR-045-26 | `unresolved_more_than_four_arguments_are_unsupported` | PASS |
| documented choice: lists, text, ties | FR-045-27 | `documented_choice_lists_text_and_ties` | PASS |
| unresolved: *.q-warning-coverage | FR-045-28 | `unresolved_no_warning_emitted` (×3) | PASS |

All 43 observed cases are covered. Every FR has at least one test, and no
test is an orphan.

## Deviations and open items

- **Identifier namespace (D-045-1), flagged.**
  - `RunMat:REGEXP:invalidOptions` follows the literal `MATLAB:x:y` →
    `RunMat:x:y` rule, so it keeps the upper-case `REGEXP` component, unlike
    the per-function `RunMat:regexp:*` ids.
  - `validatestring` uses `RunMat:validatestring:<suffix>`.
- **dec2base char bases (flagged).** The two-character char row gives
  `SecondArg`, because the observed `'ab'` is normative. All other char and
  string bases give `UnsupportedOption`.
- **No generated parent edited.** `strings/regex/mod.rs` and
  `strings/core/mod.rs` carry no "@generated" header.
- The tests call the registered builtin through the runtime dispatcher, so
  the feature was **not verified end-to-end through the VM**.
- The WASM builtin registry was not regenerated.
