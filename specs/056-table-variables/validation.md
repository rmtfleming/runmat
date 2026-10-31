# Validation Report — 056-table-variables (specs 0.2.0) — 2026-10-30

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the scratchpad cargo runner wrapper (anaconda HDF5 library path
scoped to the test process only). The target directory was shared with
other agents working on other N3/N4 features. MATLAB was not invoked.

## Freshness check (T0, T10)

In `../matlab-interface-spec`, after `git fetch -q origin`:

- Start: HEAD = origin/main = `c3305a5`. `git log 1156c97..{HEAD,origin/main}`
  over `specifications/{renamevars,rows2vars}` was empty; over
  `specifications/{varfun,convertvars}` it listed only `fc9f804` (the
  pin that added them). `git status --short` for the four spec and
  experiment directories was clean; `diff -r` against the imports: identical.
- Finish: HEAD = origin/main = `1c91aa1` (batch N5, later than both pins).
  The same `git log` over the four spec directories lists only `fc9f804`;
  `git status --short` for the spec and experiment directories is clean;
  `diff -rq` against the imported copies reports no difference. The
  imported 0.2.0 copies are current.

## Analysis (B3)

- Coverage: each of the 41 normative claims maps to an FR (`provenance.md`)
  and each FR to at least one test (table below). The 4 `*.non-table`
  claims are non-normative and back only the FR-056-40 choice.
- Consistency: identifiers agree across `spec.md` and the four descriptor
  tables: `RunMat:table:{UnrecognizedVarName,VarIndexOutOfRange,
  DuplicateVarNames,InvalidVarNames}`, `RunMat:table:renamevars:{NumNamesMismatch,RenameDim}`,
  `RunMat:table:rows2vars:CannotTransposeMulticolumnVar`,
  `RunMat:rows2vars:unrecognizedStringChoice`,
  `RunMat:table:varfun:{FunFailed,InvalidFunction}`,
  `RunMat:table:convertvars:{InvalidTextType,VariableTypesConversionFailed}`,
  `RunMat:badsubscript`, warning `RunMat:table:ModifiedVarnamesRows2Vars`,
  and `RunMat:<fn>:{InvalidInput,InvalidArgument,UnsupportedOption,
  TooManyOutputs,Internal}` for each of the four builtins.
- Every task in `tasks.md` cites an FR or SC; no orphan tests. No blocking
  findings.

## Test-first evidence (T2–T4)

The four builtin files were first written with registered stubs (each
body returned `RunMat:<fn>:Internal` "not yet implemented") and their
complete test modules, plus the shared `variable_support.rs`
(a temporary `#![allow(dead_code)]` let the stubs compile; removed with the
implementation). `cargo test -p runmat-runtime --lib -- renamevars_tests
rows2vars_tests varfun_tests convertvars_tests` → **0 passed, 72 failed**.

After the implementation the first run gave 69 passed, 3 failed:

- `varfun observed_one_var_max`: `max` called through `MappedCallable`
  returns an output list; the implementation now takes the first output
  (also for the `convertvars` function form).
- `varfun observed_table2array_of_result`: scalar results were stored as
  `Value::Num`, which `table2array` gathers into a cell; scalar double and
  logical results are now stored as 1×1 arrays.
- `convertvars observed_cellstr_to_categorical`: a test defect (RunMat's
  `cellstr` does not accept a categorical); the labels are now read with
  `categorical_labels`.

## Commands and results (T9)

- `cargo test -p runmat-runtime --lib -- renamevars_tests rows2vars_tests
  varfun_tests convertvars_tests`: **72 passed, 0 failed** (renamevars 21,
  rows2vars 19, varfun 16, convertvars 16).
- `cargo test -p runmat-runtime --lib -- builtins::table`: **360 passed,
  0 failed**.
- `rustfmt --edition 2021 --check` on the five new files: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  see "Clippy" below.

## Traceability (T10)

| claim | FR | test(s) | result |
|---|---|---|---|
| renamevars.signature-primary | FR-056-01 | `observed_one_by_name` | pass |
| renamevars.selection | FR-056-02 | `observed_one_by_name`, `observed_many_by_cellstr`, `observed_by_index_and_index_vector`, `observed_by_logical_mask`, `observed_by_string_arrays` | pass |
| renamevars.keeps-data | FR-056-03 | `observed_one_by_name`, `observed_many_by_cellstr`, `observed_row_names_kept` | pass |
| renamevars.same-name | FR-056-03 | `observed_same_name_unchanged` | pass |
| renamevars.swap | FR-056-03 | `observed_swap` | pass |
| renamevars.non-identifier | FR-056-03 | `observed_non_identifier_name_accepted` | pass |
| renamevars.error-duplicate | FR-056-04 | `observed_error_duplicate` | pass |
| renamevars.error-unknown | FR-056-04 | `observed_error_unknown` | pass |
| renamevars.error-count | FR-056-04 | `observed_error_count` | pass |
| renamevars.error-empty-name | FR-056-04 | `observed_error_empty_name` | pass |
| renamevars.error-index-range | FR-056-04 | `observed_error_index_range` | pass |
| renamevars.error-dim-Row | FR-056-04 | `observed_error_rename_row_dimension` | pass |
| rows2vars.signature-primary, rows2vars.layout | FR-056-10 | `observed_basic`, `observed_names_source_forms` | pass |
| rows2vars.default-names | FR-056-11 | `observed_basic`, `observed_row_names_name_variables`, `observed_one_row` | pass |
| rows2vars.names-source | FR-056-12 | `observed_names_source_forms` | pass |
| rows2vars.naming-modify | FR-056-13 | `observed_naming_default_modifies_and_warns`, `observed_naming_modify` | pass |
| rows2vars.warning-warn-naming-default | FR-056-13 | `observed_naming_default_modifies_and_warns` | pass |
| rows2vars.naming-preserve | FR-056-13 | `observed_naming_preserve` | pass |
| rows2vars.duplicate-source | FR-056-14 | `observed_duplicate_source_made_unique` | pass |
| rows2vars.mixed | FR-056-15 | `observed_mixed_classes_give_cells` | pass |
| rows2vars.empty-rows | FR-056-16 | `observed_empty_rows` | pass |
| rows2vars.error-multicol | FR-056-17 | `observed_error_multicolumn` | pass |
| rows2vars.error-unknown-source | FR-056-17 | `observed_error_unknown_source` | pass |
| rows2vars.error-bad-rule | FR-056-17 | `observed_error_bad_rule` | pass |
| varfun.signature-primary | FR-056-20 | `observed_mean` | pass |
| varfun.names | FR-056-20 | `observed_mean`, `observed_std`, `observed_cumsum_column_results`, `observed_one_var_max`, `observed_anonymous_function` | pass |
| varfun.rows | FR-056-21 | `observed_mean`, `observed_std`, `observed_cumsum_column_results`, `observed_anonymous_function` | pass |
| varfun.cobra-pattern | FR-056-21 | `observed_table2array_of_result`, `observed_subset` | pass |
| varfun.rownames | FR-056-22 | `observed_row_names_scalar_result` | pass |
| varfun.empty-rows | FR-056-22 | `observed_empty_rows_sum` | pass |
| varfun.error-cellstr | FR-056-23 | `observed_error_function_failed` | pass |
| varfun.error-not-function | FR-056-23 | `observed_error_not_function` | pass |
| convertvars.signature-primary | FR-056-30 | `observed_cellstr_to_string`, `observed_function_handle` | pass |
| convertvars.selection | FR-056-30 | `observed_cellstr_to_string`, `observed_string_array_selection`, `observed_index_and_mask_selection` | pass |
| convertvars.cobra-form | FR-056-30 | `observed_names_with_spaces` | pass |
| convertvars.types | FR-056-31 | `observed_cellstr_to_string`, `observed_cellstr_to_categorical`, `observed_double_to_string`, `observed_double_to_int8`, `observed_double_to_double_unchanged` | pass |
| convertvars.function | FR-056-32 | `observed_function_handle` | pass |
| convertvars.error-unknown | FR-056-33 | `observed_error_unknown_variable` | pass |
| convertvars.error-bad-type | FR-056-33 | `observed_error_bad_type` | pass |
| convertvars.error-cell-to-double | FR-056-33 | `observed_error_cell_to_double` | pass |
| *.non-table (non-normative) | FR-056-40 | `rejects_non_table*` (4 modules) | pass |
| U-056-1 outputs/arity | FR-056-40 | `unresolved_too_many_outputs` (4 modules), `rejects_*` | pass |
| U-056-2 warning coverage | FR-056-41 | `observed_naming_preserve` (no warning), none otherwise | pass |
| U-056-3 selectors | FR-056-42 | `documented_choice_selector_validation`, `documented_choice_failures_and_selection` | pass |
| U-056-4 `Row` variable | FR-056-43 | `documented_choice_row_variable_and_row_without_row_names` | pass |
| U-056-5 rename rules | FR-056-44 | `documented_choice_name_rules`, `documented_choice_properties_kept` | pass |
| U-056-6 classes | FR-056-45 | `documented_choice_classes` | pass |
| U-056-7 name rule | FR-056-46 | `documented_choice_name_rule` | pass |
| U-056-8 uniqueness | FR-056-47 | `documented_choice_uniqueness_and_row_names` | pass |
| U-056-9 options | FR-056-48 | `documented_choice_options` | pass |
| U-056-10 more args | FR-056-49 | `unresolved_more_args_unsupported` | pass |
| U-056-11 function arg | FR-056-50 | `documented_choice_function_argument` | pass |
| U-056-12 results | FR-056-51 | `documented_choice_results` | pass |
| U-056-13 type names | FR-056-52 | `documented_choice_type_names` | pass |
| U-056-14 failures | FR-056-53 | `documented_choice_failures_and_selection` | pass |

Notes on test construction:

- The `varfun` case `anonymous` (`@(v) v * 2`) is modelled as a closure
  named `anonymous#3` (the naming RunMat's compiler gives anonymous
  functions) served by an installed semantic-function invoker that doubles
  its argument, as the `cellfun` callback tests do.
- The `varfun` case `subset` (`T([1 3], 2:end)`) builds the indexed table
  `y = [4;8]` directly; table indexing itself is upstream behaviour.
- Timetable inputs (`UnsupportedOption`, FR-056-40) are implemented but not
  exercised by a test.

## Documented choices and deviations

- All open questions are handled per FR-056-40..53 (`spec.md`); none is
  asserted as MATLAB behaviour.
- **D-056-1** (storage normalisation, not a behaviour deviation): `varfun`
  stores scalar double/logical results as 1×1 arrays, because upstream
  `table2array` gathers `Value::Num` variables into a cell.
- D-034-1 (no table with rows and no variables) does not cause a wrong
  result anywhere in this feature (see `spec.md`, "Known upstream gap").
- No generated parent `mod.rs` was edited; the five `mod` lines are in the
  hand-written `table/builtins.rs`. The generated wasm registry is not
  updated (waits for the next module-composition regeneration, as F34/F44).

## Clippy

`cargo clippy -p runmat-runtime --lib --tests --message-format short`
(run twice, the second after the implementation was final): no
diagnostics in `variable_support.rs`, `renamevars.rs`, `rows2vars.rs`,
`varfun.rs`, `convertvars.rs` or `table/builtins.rs`. The run exits 101
only because of lint errors in parallel agents' in-progress files
(`plotting/ops/findall.rs` `manual_is_multiple_of`; `plotting/ops/pcolor.rs`
and `plotting/ops/surfl.rs` `unnecessary_to_owned`), which were not
touched; their errors are reported for both the lib and lib-test targets,
so both targets were linted.
