# Validation Report — 015-input-parsing

**Date**: 2026-07-22 · **Phase**: B4/B5 (implementation + validation; no
commit/push — publication requires separate approval)

## What was implemented

- `crates/runmat-runtime/src/builtins/introspection/input_parser.rs` (new):
  builtins `inputParser` (constructor → `Value::HandleObject`),
  `inputParser.addRequired`, `inputParser.addParameter`,
  `inputParser.parse`; lazy per-thread `register_class` with the three
  methods (containers.Map exemplar); state mutation via
  `runmat_gc::gc_with_value_mut` write-back (plan.md story; no side
  registry); 20 inline tests.
- `crates/runmat-runtime/src/builtins/introspection/mod.rs`: one added line
  (`pub mod input_parser;`).
- No other source file touched. Rollback = delete the new file + the one
  `mod` line.

## Verification commands (actual runs, 2026-07-22)

- `cargo check -p runmat-runtime` →
  `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 2m 38s`
  (no warnings/errors).
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- input_parser`
  (re-run after formatting) →
  `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 6635
  filtered out; finished in 0.02s`
- `cargo fmt -p runmat-runtime` then `cargo fmt --all -- --check` → clean
  (exit 0, no diff).

## Traceability (claim → FR → test → result)

| claim_id | FR | test (`builtins::introspection::input_parser::tests::…`) | observed case | result |
|---|---|---|---|---|
| inputParser.signature-primary | FR-015-01 | `normative_construct_class_is_inputparser` | construct-class | ok |
| inputParser.results-fields | FR-015-01 | `normative_construct_class_is_inputparser` | construct-class | ok |
| inputParser.results-fields | FR-015-02 | `normative_required_chain_results_struct_and_value` | results-class, required-value | ok |
| inputParser.results-fields | FR-015-03 | `normative_param_supplied_overrides_default` | param-value | ok |
| inputParser.results-fields | FR-015-04 | `normative_param_default_used_when_omitted` | param-default | ok |
| inputParser.using-defaults | FR-015-05 | `normative_using_defaults_cell_1x1` | using-defaults | ok |
| results-fields + using-defaults | FR-015-02..05 | `normative_combined_chain_required_and_parameters` | (composite) | ok |
| results-fields (stateful chains) | FR-015-06 | all normative chains (no reassignment) + `unresolved_choice_alias_sees_mutations` | multi-step cases | ok |
| (RunMat dispatch, not a MATLAB claim) | FR-015-07 | `dual_syntax_method_call_via_call_method`, `dual_syntax_function_call_via_dispatcher` | — | ok |
| carried unresolved (q-class-semantics, q-methods) | FR-015-08 | `unresolved_choice_validator_argument_rejected`, `unresolved_choice_unknown_parameter_errors`, `unresolved_choice_case_sensitive_parameter_names`, `unresolved_choice_missing_required_errors`, `unresolved_choice_dangling_name_value_errors`, `unresolved_choice_duplicate_registration_rejected`, `unresolved_choice_duplicate_parameter_in_parse_rejected`, `unresolved_choice_constructor_rejects_inputs`, `unresolved_choice_pre_parse_properties`, `unresolved_choice_using_defaults_lists_name_as_char` | — | ok |
| SC-015-2 discoverability | — | `registration_discoverable_class_and_builtins` | — | ok |

All 20 tests pass; every FR has ≥1 covering test; no orphan tests (each
test cites an FR or SC in its doc comment).

## Test-tier register

- `normative_*` (6): observed cases, claim-cited.
- `dual_syntax_*` (2): FR-015-07 dispatch coverage (RunMat semantics, not
  MATLAB-conformance assertions).
- `unresolved_choice_*` (11): documented independent choices / explicit
  RunMat errors, never asserted MATLAB-conformant.
- `registration_discoverable_*` (1): SC-015-2.
- No `summary_derived` tier exists for this feature (no summary-only FRs).

## Deviations and notes for the reviewer

1. **`class` return representation**: RunMat's `class` builtin returns its
   name via `Value::String` (pre-existing convention shared by all
   builtins), while the observation records char 1×11. Tests assert the
   normative content ('inputParser', length 11 / 'struct', length 6); the
   char-vs-string representation of `class`'s own return value is a
   pre-existing RunMat-wide matter outside this feature's scope.
2. **Internal schema properties** (`__ip_required__`, `__ip_param_names__`,
   `__ip_param_defaults__`) are visible through generic field access
   (`fieldnames`/`getfield`) — documented cosmetic leak (plan.md caveat);
   `properties()`-surface behaviour is unobserved in the export.
3. **Unimplemented-by-design** (unresolved in the approved export):
   validation functions (explicit `RunMat:inputParser:ValidatorsUnsupported`
   error), `addOptional` (not registered → undefined-function error),
   `KeepUnmatched`, `StructExpand`, `CaseSensitive`, `Unmatched` (absent);
   parameter matching exact/case-sensitive by documented choice.
4. **Thread-locality**: class registration is per-thread and lazy in the
   constructor (containers.Map pattern) — pre-existing machinery
   limitation, not widened here.
5. **Clean-room**: MATLAB was not invoked at any point; all behaviour
   derives from the imported export at
   `specs/imports/matlab-interface-spec/specifications/inputParser/`
   (commit pin `b054f3ad809ceee28529a4dd7a6e28d2b6b434ef`). Independent
   legal review MAY still be required (Constitution X).
6. **Observed during validation (not caused by this feature)**:
   `specs/imports/matlab-interface-spec/SHA256SUMS` shows as modified in
   the working tree while sibling batch-3 features were implemented
   concurrently. Import-area files are read-only reference material per
   `IMPORT-MANIFEST.md`; flagged for the maintainer to reconcile before
   any commit.

## Task completion

T1–T8 complete (T2/T3 tests authored with the implementation gated behind
them in the same module; red state exercised by stubbing during
development, final state green as recorded above).
