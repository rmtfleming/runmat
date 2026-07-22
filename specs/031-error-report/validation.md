# Validation Report — 031-error-report — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (Finished, 16.36s after sibling
  build-lock wait)
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::diagnostics::get_report`
  — **8 passed, 0 failed** (6872 filtered out):
  `descriptor_and_spec_metadata_consistent`,
  `get_report_is_registered`,
  `get_report_type_is_string`,
  `observed_message_report_is_char_row`,
  `observed_output_class_char_row`,
  `unresolved_choice_empty_message`,
  `unresolved_choice_non_mexception_input_errors`,
  `unresolved_choice_report_is_message_only` — all `ok`
- `cargo fmt` then `cargo fmt --all -- --check` — clean
- `cargo clippy -p runmat-runtime --lib` — clean (Finished, no warnings)

## Traceability (claim → FR → test → result)

| Claim | FR | Test (`builtins::diagnostics::get_report::tests::…`) | Result |
|---|---|---|---|
| getReport.signature-primary, getReport.char-report, getReport.output-class | FR-031-01 | `observed_message_report_is_char_row` | PASS |
| getReport.char-report, getReport.output-class | FR-031-02 | `observed_message_report_is_char_row`, `observed_output_class_char_row` | PASS |
| getReport.q-options (unresolved → message-only) | FR-031-03 | `unresolved_choice_report_is_message_only`, `unresolved_choice_empty_message` | PASS (non-normative) |
| (documented choice — input-type error path) | FR-031-04 | `unresolved_choice_non_mexception_input_errors` | PASS (non-normative) |
| getReport.q-options (unresolved → options omitted) | FR-031-05 | `descriptor_and_spec_metadata_consistent` (Fixed single output, 1 signature, 1 error) | PASS |
| — | SC-031-2 | `get_report_is_registered`, `get_report_type_is_string` | PASS |

The observed case reproduces exactly:
`getReport(MException('x:y','oops'))` → `'oops'`, class char, size 1×4
(`CharArray` rows 1, cols 4).

## Deviations and notes

- Placement: `getReport` lives in `crates/runmat-runtime/src/builtins/diagnostics/`
  beside the other MException-operating builtins `throw`/`rethrow`, rather than
  in `introspection/`. The file is named `getReport.rs` (matching the source
  function's camelCase) and attached with the house `#[path = "…"]` convention
  under a snake-case module `get_report`; the Rust module/test path is
  therefore `builtins::diagnostics::get_report::…` (note the case difference
  from the function name `getReport`).
- Report content is message-only for every `MException` (FR-031-03). The
  identifier, a populated stack, and the `'extended'`/`'basic'`/hyperlink
  options are unobserved (`getReport.q-options`) and are NOT rendered —
  documented independent choices, tagged `unresolved_choice` in tests and NOT
  asserted MATLAB-conformant.
- An empty message returns a 1×0 char row; a non-`MException` input is rejected
  with `RunMat:getReport:InvalidInput` — both documented choices
  (FR-031-03/04), unobserved.
- The `'extended'`/`'basic'` and hyperlink options are omitted (descriptor
  output mode Fixed, single one-argument signature), per unresolved question
  `getReport.q-options` (FR-031-05).
- Registration is via the `#[runtime_builtin]` inventory macro with
  `builtin_path` (native + WASM registries), verified through
  `builtin_function_by_name("getReport")`.

## Unresolved (returned to specification side)

`getReport.q-options` (the `'extended'`/`'basic'` format selectors and
hyperlink handling); observation coverage for the fuller multi-line report
(identifier formatting and stack trace), for empty-message inputs, and for the
input-type error path.
