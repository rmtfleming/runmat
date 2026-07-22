# Validation Report — 033-environment-identity — 2026-07-22

## Binding note — RunMat-honest identity

MATLAB's observed identity values (`version` =
`'26.1.0.3276743 (R2026a) Update 3'`, `version('-release')` = `'2026a'`, the
MATLAB `ver` product struct, `verLessThan('matlab','1.0')` = `false`,
`license('test','MATLAB')` = `1`-as-MATLAB-state) are **NON-NORMATIVE** and are
**deliberately not reproduced** by RunMat (Constitution Principle II / honesty;
see `spec.md` gate decision). RunMat returns its own honest identity. No test
below asserts a MATLAB observed identity value as RunMat behaviour; the
`contract_*` tests assert only class/shape plus RunMat's own values, and
several tests assert the divergence explicitly.

## Commands and results

- `cargo check -p runmat-runtime` — clean (`Finished` in 1m 07s, after waiting
  out a concurrent sibling's build lock and an unrelated `what.rs` dead-code
  error that a sibling cleared; no edits made to sibling files).
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::introspection::ver builtins::introspection::license`
  — **35 passed, 0 failed** (6941 filtered out), finished in 0.01s:
  - `version::tests`: `contract_no_arg_returns_runmat_char_version`,
    `contract_release_option_returns_char_release_tag`,
    `documented_choice_release_option_case_insensitive_and_string`,
    `documented_choice_unrecognized_option_errors`,
    `documented_choice_non_text_option_errors`,
    `documented_choice_too_many_inputs_errors`, `version_is_registered`,
    `descriptor_and_spec_metadata_consistent`, `version_type_is_string`
  - `ver::tests`: `contract_no_arg_returns_runmat_struct`,
    `contract_runmat_product_returns_struct`,
    `documented_choice_matlab_product_is_empty_struct_array`,
    `documented_choice_unknown_product_is_empty_and_case_insensitive`,
    `documented_choice_invalid_inputs_error`, `ver_is_registered`,
    `descriptor_and_spec_metadata_consistent`, `ver_type_is_struct`
  - `ver_less_than::tests`: `contract_older_than_high_version_is_true`,
    `contract_runmat_version_below_one_is_true_diverging_from_matlab`,
    `contract_comparisons_reflect_runmat_version`,
    `documented_choice_toolbox_name_maps_to_runmat_version`,
    `documented_choice_version_less_than_semantics`,
    `documented_choice_invalid_inputs_error`, `verlessthan_is_registered`,
    `descriptor_and_spec_metadata_consistent`, `verlessthan_type_is_bool`
  - `license::tests`: `contract_test_returns_double_one`,
    `contract_test_any_feature_is_available`,
    `documented_choice_no_arg_returns_runmat_identity`,
    `documented_choice_inuse_is_empty_struct_array`,
    `documented_choice_checkout_checkin_succeed`,
    `documented_choice_invalid_usage_errors`, `license_is_registered`,
    `descriptor_and_spec_metadata_consistent`, `license_type_is_num`
  — all `ok`.
- `cargo fmt` then `cargo fmt --all -- --check` — clean (exit 0).
- `cargo clippy -p runmat-runtime --lib` — clean (`Finished`, no warnings in
  the four new files).

## Actual RunMat outputs (asserted by the passing tests)

| call | RunMat output | class/shape | MATLAB observed (NON-NORMATIVE, not returned) |
|---|---|---|---|
| `version` | `'0.5.6 (RunMat)'` | char row | `'26.1.0.3276743 (R2026a) Update 3'` |
| `version('-release')` | `'R2026a-compat'` | char row | `'2026a'` |
| `ver` / `ver('RunMat')` | struct `Name='RunMat', Version='0.5.6', Release='R2026a-compat', Date='2026-06-28'` | struct 1×1 | MATLAB product struct |
| `ver('MATLAB')` | empty struct array (0×1) | — | 1×1 MATLAB struct |
| `verLessThan('matlab','99.0')` | `true` | logical 1×1 | `true` |
| `verLessThan('matlab','1.0')` | **`true`** (0.5.6 < 1.0) | logical 1×1 | `false` ← **divergence** |
| `license('test','MATLAB')` | `1` | double 1×1 | `1` |

## Traceability (claim → FR → test → result)

| Claim (contract part) | FR | Test (`builtins::introspection::…::tests::…`) | Result |
|---|---|---|---|
| version.signature-primary, version.output-class; version.version-strings (**shape only**) | FR-033-01 | `version::contract_no_arg_returns_runmat_char_version` | PASS (contract) |
| version.output-class (`-release` shape) | FR-033-02 | `version::contract_release_option_returns_char_release_tag`, `documented_choice_*` | PASS (contract) |
| ver.signature-primary, ver.output-class; ver.product-struct (**shape only**) | FR-033-03 | `ver::contract_no_arg_returns_runmat_struct`, `contract_runmat_product_returns_struct` | PASS (contract) |
| (documented choice — RunMat ≠ MATLAB) | FR-033-04 | `ver::documented_choice_matlab_product_is_empty_struct_array`, `documented_choice_unknown_product_is_empty_and_case_insensitive`, `documented_choice_invalid_inputs_error` | PASS (non-normative) |
| verLessThan.signature-primary, verLessThan.output-class; verLessThan.logical-compare (**semantics only**) | FR-033-05 | `ver_less_than::contract_older_than_high_version_is_true`, `contract_runmat_version_below_one_is_true_diverging_from_matlab`, `contract_comparisons_reflect_runmat_version`, `documented_choice_version_less_than_semantics`, `documented_choice_toolbox_name_maps_to_runmat_version`, `documented_choice_invalid_inputs_error` | PASS (contract + divergence) |
| license.signature-primary, license.output-class, license.test-numeric (`1`==available, RunMat truth) | FR-033-06 | `license::contract_test_returns_double_one`, `contract_test_any_feature_is_available` | PASS (contract) |
| (documented choice — other license forms) | FR-033-07 | `license::documented_choice_no_arg_returns_runmat_identity`, `documented_choice_inuse_is_empty_struct_array`, `documented_choice_checkout_checkin_succeed`, `documented_choice_invalid_usage_errors` | PASS (non-normative) |
| — (registration/metadata) | SC-033-2 | `*_is_registered`, `descriptor_and_spec_metadata_consistent`, `*_type_is_*` (all four modules) | PASS |

## MATLAB-value-claims deliberately NOT asserted (honesty)

The following observed values are documented on the specification side and are
**explicitly not claimed as satisfied** by RunMat:

- `version.version-strings` **value** `'26.1.0.3276743 (R2026a) Update 3'` /
  `'2026a'` — RunMat returns `'0.5.6 (RunMat)'` / `'R2026a-compat'`.
- `ver.product-struct` **MATLAB contents** — RunMat returns its own struct;
  `ver('MATLAB')` is empty.
- `verLessThan.logical-compare` **values** `true`/`false` for MATLAB 26.x —
  RunMat computes its own results; `verLessThan('matlab','1.0')` is `true`.
- `license.test-numeric` `1` as MATLAB's *licensing state* — RunMat's `1` is
  its own open-source availability, not a copy.

Honesty guards in the tests actively reject MATLAB's identity leaking into
RunMat output (`!contains("R2026a) Update")`, `Version != '26.1.0.3276743 …'`,
`release != '2026a'`).

## Deviations and notes

- `verLessThan.rs` keeps the export's camelCase filename; the Rust module is
  `ver_less_than` via `#[path = "verLessThan.rs"]` (mirrors `getReport.rs` →
  `get_report`) to satisfy `non_snake_case` cleanly. Builtin name registered as
  `"verLessThan"`.
- `ver('MATLAB')`, `ver(unknown)`, and `license('inuse')` use RunMat's empty
  struct-array representation (`CellArray::new(vec![],0,1)`, i.e. `Value::Cell`
  0×1), the same representation `dir` uses for "no matches".
- Registration via the `#[runtime_builtin]` inventory macro with `builtin_path`
  (native + WASM registries), verified through `builtin_function_by_name`.
- Concurrency: a sibling `what.rs` dead-code error and build lock were waited
  out (not edited), per the programme rule; siblings' additions to
  `introspection/mod.rs` (`methods`, `properties`) merged cleanly.

## Unresolved (returned to specification side)

`version.q-values`; `ver.q-fields`, `ver.q-install-dependent`;
`verLessThan.q-deprecated`; `license.q-other-actions`, `license.q-value`. In
every case RunMat substitutes its own honest identity/behaviour rather than
MATLAB's observed values.
