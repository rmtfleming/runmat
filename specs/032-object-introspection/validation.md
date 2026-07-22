# Validation Report — 032-object-introspection — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (Finished dev profile).
- `cargo fmt --all -- --check` — clean (exit 0).
- `cargo clippy -p runmat-runtime --lib` — clean (no warnings in the three
  new files).
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::introspection::properties builtins::introspection::methods what`
  — **30 passed, 0 failed, 0 ignored** (6946 filtered out).

### Observed test output (excerpt)

```
running 30 tests
test builtins::introspection::methods::tests::observed_input_parser_methods_is_cellstr_column ... ok
test builtins::introspection::methods::tests::unresolved_choice_method_count_not_asserted ... ok
test builtins::introspection::properties::tests::observed_input_parser_properties_is_cellstr_column ... ok
test builtins::introspection::properties::tests::unresolved_choice_property_count_not_asserted ... ok
test builtins::io::repl_fs::what::tests::observed_what_returns_scalar_struct ... ok
test builtins::io::repl_fs::what::tests::observed_what_struct_has_documented_fields ... ok
test builtins::io::repl_fs::what::tests::unresolved_choice_classifies_files_and_folders ... ok
...
test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 6946 filtered out
```

(10 tests per builtin × 3 builtins = 30.)

## Traceability (claim → FR → test → result)

| Claim(s) | FR | Test | Result |
|---|---|---|---|
| properties.signature-primary, properties.cellstr-names | FR-032-01 | `properties::tests::observed_input_parser_properties_is_cellstr_column`, `descriptor_exposes_primary_signature` | PASS |
| methods.signature-primary, methods.cellstr-names | FR-032-02 | `methods::tests::observed_input_parser_methods_is_cellstr_column`, `descriptor_exposes_primary_signature` | PASS |
| what.signature-primary, what.output-class, what.folder-struct | FR-032-03 | `what::tests::observed_what_returns_scalar_struct`, `observed_what_struct_has_documented_fields`, `descriptor_exposes_primary_signature` | PASS |
| (value non-assertion) | FR-032-01/02 | `properties::…::unresolved_choice_property_count_not_asserted`, `methods::…::unresolved_choice_method_count_not_asserted` | PASS (non-normative guard) |
| unresolved choices — members/forms | FR-032-04 | `properties::…::{unresolved_choice_no_properties_is_empty_column, class_name_text_form, private_properties_excluded, unknown_class_name_errors, non_object_input_errors}`; `methods::…::{unresolved_choice_no_methods_is_empty_column, class_name_text_form, private_methods_excluded, unknown_class_name_errors, non_object_input_errors}` | PASS (non-normative) |
| unresolved choices — `what` contents/forms | FR-032-04 | `what::…::{unresolved_choice_classifies_files_and_folders, empty_folder_lists_are_empty, no_argument_scans_current_directory, too_many_inputs_errors, non_text_argument_errors}` | PASS (non-normative) |
| SC-032-2 registration | — | `{properties,methods,what}::tests::registration_discoverable_by_name`, `gpu_and_fusion_specs_match_name` | PASS |

## Value non-assertion (explicit)

Per `spec.md` "Value non-assertion", the following observed values are
deliberately **not** asserted:

- `properties(inputParser)` observed **9×1** — the count and the specific
  member names are MATLAB-object-specific. The normative test asserts only
  class `cell`, N×1 column shape, and all-elements-char. RunMat's own
  `inputParser` reports `Results` + `UsingDefaults` (its feature-015
  `PropertyDef`s), and `unresolved_choice_property_count_not_asserted`
  explicitly asserts the count is *not* 9.
- `methods(inputParser)` observed **19×1** — likewise not asserted;
  RunMat reports `addRequired`, `addParameter`, `parse`, and
  `unresolved_choice_method_count_not_asserted` asserts the count is *not* 19.
- `what(tempdir)` observed **1×1 struct** — the struct class and 1×1 size are
  asserted, plus the presence of the documented fields (`m`, `mat`, `mex`,
  `mlx`, `mlapp`, `classes`, `packages`, `path`). The listed contents are
  environment-specific and are asserted *only* in the fully controlled
  temp-folder classification test, never for an uncontrolled folder
  (`what.q-fields`, `what.q-content` remain unresolved).

## Deviations and notes

- **`what` placement**: implemented in `io/repl_fs/what.rs` (filesystem
  category) rather than `introspection/`, because it scans folders through
  `runmat_filesystem` exactly like `dir`/`ls`. Justified in `plan.md`.
- **Member source**: `properties`/`methods` read RunMat's class registry
  (`ClassDef` public members over the inheritance chain), not dynamic instance
  properties — this is what makes `properties` differ from `fieldnames` and
  yields the clean declared-member list for RunMat's `inputParser`.
- **Ordering / public-only filter**: ascending order and the public-access
  filter are documented independent choices (declaration order and hidden
  members are unresolved in the exports); not asserted as MATLAB order.
- **`methods('-full')`**: unobserved; intentionally not implemented.
- Registration is via the compile-time `#[runtime_builtin]` inventory macro
  (the mechanism used by every existing builtin); discoverability is verified
  by the `registration_discoverable_by_name` tests.
- No MATLAB was invoked at any point (Constitution II).

## Unresolved (returned to specification side)

`properties.q-class-dependent`, `methods.q-class-dependent`, `methods.q-full`,
`what.q-fields`, `what.q-content`.
