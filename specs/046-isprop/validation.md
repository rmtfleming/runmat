# Validation Report — 046-isprop — 2026-10-30

Environment: same as `specs/045-text-conversion/validation.md` (same
session, shared target directory, runner wrapper, MATLAB not invoked).

## Freshness

The check ran in `../matlab-interface-spec`, read-only, both at the start
and at the finish (2026-10-30T01:28Z):

- `git fetch -q --all` exited 0.
- HEAD = origin/main = `8f72edb`.
- `git diff --name-only c65e238 {HEAD,origin/main} -- specifications/isprop`
  returned nothing.
- `git status --short` was clean.

## Claim-id check

All 7 cited `isprop.*` claim ids are present in the imported
`provenance.yaml`. `isprop.q-warning-coverage` and `isprop.q-case-objects`
are present in the imported `interface.yaml`.

## Test-first evidence (T4)

Setup:
- `isprop.rs` held only its test module.
- `pub mod isprop;` was registered in the generated `introspection/mod.rs`.

The same run as F45 gave **0 passed, 74 failed**. The 16 isprop tests were
all among the failures. Every failure was `RunMat:UndefinedFunction`, with 10
"Undefined function: isprop" occurrences.

That run also showed that **`MException` is not a callable RunMat builtin**
("Undefined function: MException", 5 occurrences). The tests therefore build
`Value::MException(MException::new("a:b", "c"))` directly. This is a test
construction choice and does not change the asserted behaviour. The
`string-name` case uses a string-scalar property name on that value, and the
observed constructor arguments affect nothing that isprop reads.

## Commands and results (T6)

- **Feature tests.** `cargo test -p runmat-runtime --lib -- … isprop_tests`
  (combined with F45) gave **74 passed, 0 failed**, including all **16**
  isprop tests:
  - observed (11): `observed_figure_has_visible`,
    `observed_figure_lacks_unknown`, `observed_figure_name_case_insensitive`,
    `observed_struct_field_is_not_a_property`,
    `observed_table_properties_is_not_a_property`,
    `observed_table_variable_is_not_a_property`, `observed_number_is_false`,
    `observed_mexception_has_message`, `observed_mexception_lacks_unknown`,
    `observed_handle_array_gives_logical_row`,
    `observed_string_name_accepted`.
  - other (5): `documented_choice_arity_and_name_forms`,
    `unresolved_case_mismatch_on_objects_is_unsupported`,
    `documented_choice_non_objects_are_false`,
    `documented_choice_class_objects_and_arrays`,
    `unresolved_no_warning_emitted`.
- **Introspection regressions.**
  `cargo test -p runmat-runtime --lib -- introspection` gave **419 passed,
  7 failed** (426 tests, including the 16 new ones). All 7 failures are
  upstream or environmental, not caused by this feature:
  - `debugging::tests::getcallinfo_reports_current_frame` already failed in
    the pre-change baseline.
  - `which::tests::{which_reports_builtin, which_all_returns_cell_array,
    which_parses_leading_option, which_allows_uppercase_and_repeated_flags,
    which_accepts_char_and_string_array_inputs}` report "'sin' not found".
  - `memoize::tests::requested_output_count_is_part_of_cache_key` is flaky.
    Another run had 8 memoize failures.
- **Attribution check.**
  - The four F45/F46 `pub mod` lines were removed temporarily, and
    `-- introspection::which introspection::memoize introspection::debugging`
    was rerun.
  - Result: **8 failed**, the same seven plus one more flaky memoize test.
    So the failures happen without this feature too.
  - The lines were then restored, and the diff was checked.
- **Baseline note.** The pre-change baseline run of
  `-- builtins::strings introspection` **hung**. ~36 upstream introspection
  tests (`on_cleanup`, `memoize`, `input_parser`, `dynamicprops`, `methods`,
  `clearvars`, `object_serialization`) sat at 0% CPU for more than 8 minutes,
  and the run was killed. Post-change runs of `-- introspection` completed.
- **Formatting.** `rustfmt --edition 2021 --check …/introspection/isprop.rs`
  is clean.
- **Clippy.**
  - The first run flagged three `unnecessary to_string` uses in `isprop.rs`,
    and they were fixed (`class_name.as_ref()`).
  - The rerun reports no diagnostics in `isprop.rs`. The remaining errors
    are all in other agents' new files (see the F45 report).

## Traceability (claim → FR → test → result)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| isprop.signature-primary, isprop.output-class | FR-046-01 | `observed_figure_has_visible`, `observed_number_is_false` | PASS |
| isprop.objects | FR-046-02 | `observed_figure_has_visible`, `observed_figure_lacks_unknown`, `observed_mexception_has_message`, `observed_mexception_lacks_unknown` | PASS |
| isprop.case-insensitive-graphics | FR-046-03 | `observed_figure_name_case_insensitive` | PASS |
| isprop.non-objects | FR-046-04 | `observed_struct_field_is_not_a_property`, `observed_table_properties_is_not_a_property`, `observed_table_variable_is_not_a_property`, `observed_number_is_false` | PASS |
| isprop.arrays | FR-046-05 | `observed_handle_array_gives_logical_row` | PASS |
| isprop.string-name | FR-046-06 | `observed_string_name_accepted` | PASS |
| documented choice: arity, name forms | FR-046-07 | `documented_choice_arity_and_name_forms` | PASS |
| unresolved: isprop.q-case-objects | FR-046-08 | `unresolved_case_mismatch_on_objects_is_unsupported` | PASS (see deviation) |
| documented choice: non-objects, timetable | FR-046-09 | `documented_choice_non_objects_are_false` | PASS |
| documented choice: object arrays, ClassRef, registry rules | FR-046-10, 11 | `documented_choice_class_objects_and_arrays` | PASS |
| unresolved: isprop.q-warning-coverage | FR-046-12 | `unresolved_no_warning_emitted` | PASS |

All 11 observed cases are covered. Every FR has at least one test, and no
test is an orphan.

## Deviations and open items

- **Generated parent edited (flag for the maintainer README).**
  - `pub mod isprop;` was added to
    `crates/runmat-runtime/src/builtins/introspection/mod.rs`, which is
    marked "@generated … Do not edit".
  - This follows the `strmatch`/`findstr` precedent.
  - The line will be dropped if the module composition is regenerated.
- **Dispatcher quirk affects FR-046-08 for class objects.**
  - Called directly, `isprop_builtin(obj, 'alpha')` raises
    `RunMat:isprop:UnsupportedOption`.
  - Through `call_builtin_async`, the dispatcher retries the duplicate
    registration after that error, and the retry returns `false` for a
    `Value::Object` receiver.
  - The MException case raises `UnsupportedOption` through both paths.
  - The test asserts the error on the direct call and records the
    dispatcher's `false`, so an upstream fix will show up. This is the
    known upstream dispatcher issue, and it was not fixed here (out of
    scope).
- **Timetable check is conditional.**
  `documented_choice_non_objects_are_false` checks a timetable only when
  `timetable(seconds(1), 2)` constructs successfully.
- **Graphics coverage** is limited to the handles RunMat's `get` resolves.
  Zoom, pan and data-cursor mode objects are not consulted.
- The tests call the registered builtin through the runtime dispatcher, so
  the feature was **not verified end-to-end through the VM**.
- The WASM builtin registry was not regenerated.
