# Validation Report — 052-data-n2 (specs 0.2.0) — 2026-10-30

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the cargo runner wrapper (anaconda `lib/` on `LD_LIBRARY_PATH` for
the test process only). The target directory was shared with other agents
working on other N2 features. MATLAB was not invoked.

## Freshness check (T0, T9)

In `../matlab-interface-spec`, after `git fetch -q origin`: `HEAD` =
`origin/main` = `1156c97` (batch N3, later than the pin `8f72edb`);
`git diff --name-only 8f72edb HEAD` and `… 8f72edb origin/main` over
`specifications/{addnode,categories,textread}` were empty, and
`git status --short` for them was clean. Checked at the start and again at
the end of the work; the imported 0.2.0 copies are current.

## Analysis (B3)

- Coverage: every normative claim (23) maps to an FR (provenance.md) and
  every FR to at least one test (table below). The one non-normative claim
  (categories.non-categorical) backs only U-052-3.
- Consistency: error identifiers in spec.md, provenance.md and the three
  descriptor tables agree (`RunMat:addnode:{NonUniqueNames,InvalidNrNodes,
  InvalidInput,UnsupportedOption,UndefinedFunction,TooManyOutputs,Internal}`,
  `RunMat:categories:{InvalidInput,UnsupportedOption,UndefinedFunction,
  TooManyOutputs,Internal}`, `RunMat:textread:{FileNotFound,InvalidInput,
  UnsupportedOption,TooManyOutputs,Internal}`).
- Every task in tasks.md cites an FR or SC; no orphan tests. No blocking
  findings.

## Test-first evidence (T2–T4)

The three files were first written with the builtins registered as stubs
(each returned `RunMat:<fn>:Internal` "not yet implemented") and the
complete test modules. `cargo test -p runmat-runtime --lib --
addnode_tests categories_tests textread_tests` →
**0 passed, 36 failed** (addnode 12, categories 10, textread 14). (The first
attempt could not compile because of another agent's in-progress
`plotting/ops/alpha.rs`; the run was taken once the shared build compiled.)

## Commands and results (T8)

- Feature tests after the implementation: **36 passed, 0 failed**. The
  first post-implementation run had one failure in this feature's own test
  `unresolved_choice_textread_field_data` (the test requested 2 outputs for
  a one-field format, so `TooManyOutputs` correctly preceded the data
  check); the test was corrected to request 1 output, and the re-run was
  36/36.
- `cargo test -p runmat-runtime --lib -- builtins::graph builtins::table
  builtins::io`: **1499 passed, 29 failed, 2 ignored**. All 29 failures are
  in `builtins::io::{display, mat::matfile, repl_fs::*}` and none in graph
  or table. They are not caused by this feature: with `--test-threads=1
  --skip textread_tests` the same tests still fail (1120 passed, 26
  failed), the failures are `PoisonError` on a shared test lock poisoned by
  an earlier failing io test, and `builtins::io::repl_fs::file_transfer::
  copyfile` run alone passes (9/9). Recorded here as pre-existing or
  other-agent state, not attributable to F52.
- `rustfmt --edition 2021 --check` on the three files: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`: no
  findings in this feature's files; the run's only finding is
  `clippy::cloned_ref_to_slice_refs` in another agent's
  `builtins/containers/map/conformance_tests.rs:303` (not touched).

## Traceability (claim → FR → test → result) (T9)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| addnode.signature-primary, addnode.count, addnode.class | FR-052-01 | `observed_addnode_count_and_zero`, `observed_addnode_digraph`, `observed_addnode_empty_graph` | PASS |
| addnode.names | FR-052-02 | `observed_addnode_names`, `observed_addnode_names_to_empty_graph` | PASS |
| addnode.names-to-unnamed | FR-052-03 | `observed_addnode_names_to_unnamed` | PASS |
| addnode.error-dup-name | FR-052-04 | `rejects_addnode_duplicate_name` | PASS |
| addnode.error-negative | FR-052-04 | `rejects_addnode_negative_count` | PASS |
| categories.signature-primary, categories.output-class, categories.sorted-default | FR-052-05, FR-052-06 | `observed_categories_from_cellstr_sorted`, `observed_categories_case_and_string_source` | PASS |
| categories.explicit-order | FR-052-06 | `observed_categories_explicit_order` | PASS |
| categories.numeric | FR-052-06 | `observed_categories_numeric_source` | PASS |
| categories.missing | FR-052-07 | `observed_categories_empty_text_not_a_category` | PASS |
| categories.empty | FR-052-07 | `observed_categories_empty_categorical` | PASS |
| textread.signature-primary, textread.columns | FR-052-08 | `observed_textread_numeric_columns`, `observed_textread_single_column` | PASS |
| textread.types | FR-052-09 | `observed_textread_mixed_types`, `observed_textread_numeric_columns` | PASS |
| textread.options | FR-052-10 | `observed_textread_options` | PASS |
| textread.skip | FR-052-11 | `observed_textread_skip_field` | PASS |
| textread.default-format | FR-052-12 | `observed_textread_default_format` | PASS |
| textread.empty | FR-052-13 | `observed_textread_empty_file` | PASS |
| textread.no-warning | FR-052-14 | `observed_textread_numeric_columns` (warning store empty after the call) | PASS |
| textread.error-missing | FR-052-15 | `rejects_textread_missing_file` | PASS |
| (design) new object, counts consistent, input unchanged | FR-052-16 | every `observed_addnode_*` checks `numnodes`/`numedges` via the public builtins; `observed_addnode_count_and_zero` checks the input is unchanged | PASS |
| U-052-1 | FR-052-17 | `unresolved_choice_addnode_output_count`, `unresolved_choice_categories_output_count`, `unresolved_choice_addnode_arity_and_input_class`, `unresolved_choice_categories_arity`, `unresolved_choice_textread_options` (empty args) | PASS |
| U-052-2 | FR-052-17 | `unresolved_choice_addnode_invalid_counts`, `unresolved_choice_addnode_names_and_forms`, `unresolved_choice_addnode_arity_and_input_class` | PASS |
| categories.non-categorical (non-normative), U-052-3 | FR-052-17 | `unresolved_choice_categories_non_categorical`, `unresolved_choice_categories_arity`, `unresolved_choice_categories_shapes` | PASS |
| U-052-4 | FR-052-17 | `unresolved_choice_textread_format_grammar` | PASS |
| U-052-5 | FR-052-17 | `unresolved_choice_textread_field_data` | PASS |
| U-052-6 | FR-052-17 | `unresolved_choice_textread_options` | PASS |
| U-052-7 | FR-052-17 | `unresolved_choice_textread_default_format`, `unresolved_choice_textread_field_data` (empty `%s`) | PASS |
| U-052-8 | FR-052-17 | `unresolved_choice_textread_output_count`, `observed_textread_single_column` (dispatcher, no count) | PASS |
| U-052-9 | FR-052-17 | `unresolved_choice_textread_unreadable_path`, `rejects_textread_missing_file` | PASS |
| *.q-warning-coverage, addnode.q-graph-variables, textread.q-more-formats | FR-052-18 | no warning path in the implementation; graph-variable and format questions via U-052-2/4/6 tests | covered via U tests / not tested (warnings) |

Every FR maps to at least one test; every test cites a claim or a
documented choice (no orphan tests).

## Unresolved-question dispositions

| id | disposition |
|---|---|
| addnode.q-warning-coverage, categories.q-warning-coverage | no warnings emitted |
| addnode.q-graph-variables | other node inputs raise `RunMat:addnode:UnsupportedOption` (U-052-2) |
| textread.q-more-formats | only `%f`, `%d`, `%s`, `%*s`, `headerlines`, `delimiter`; others `RunMat:textread:UnsupportedOption` (U-052-4, U-052-6) |

## Deviations

None. The addnode observations construct graphs only with `graph(s, t)`,
`digraph(s, t)` and `graph()`, so the upstream constructor gaps D-036-1 /
D-043-1 (edges kept in input order) and D-036-2 (`graph(s, t, w, n)` not
accepted) do not affect any observed case; `addnode` never reorders edges.
RunMat's `categorical` constructor produced the observed category order and
empty-text handling for every observed case.

## Notes

- Integer-capability metadata was not declared (as instructed).
- No generated parent `mod.rs` was edited. The three registrations are in
  hand-written parents: `mod addnode;` in `graph/domain/mod.rs`,
  `mod categories;` in `table/builtins.rs`, `pub mod textread;` in
  `io/filetext/mod.rs`. Wasm registration of `addnode` (re-exported from
  the generated `graph/mod.rs` list) waits for the next module-composition
  regeneration, as for features 036/043.
- No MathWorks material was consulted; MATLAB was not invoked.
  Independent legal review MAY still be required.
