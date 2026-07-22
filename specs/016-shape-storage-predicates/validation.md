# Validation Report — 016-shape-storage-predicates — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (`Finished \`dev\` profile ... in 16.69s`)
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::logical::tests::isrow builtins::logical::tests::iscolumn builtins::logical::tests::issparse builtins::logical::tests::iscellstr builtins::logical::tests::istable` — **31 passed, 0 failed** (`test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 6735 filtered out`)
- `cargo fmt -p runmat-runtime` then `cargo fmt --all -- --check` — clean

Test counts per module: isrow 7 (4 observed + 3 unresolved_choice),
iscolumn 7 (4 + 3), issparse 5 (3 + 2), iscellstr 8 (4 + 4), istable 4
(2 observed + 2 unresolved_choice).

## Traceability (claim → FR → test → result)

| Claim | FR | Test | Result |
|---|---|---|---|
| isrow.signature-primary, output-class, logical-row-test | FR-016-01 | `isrow::tests::observed_row_vector_is_true`, `observed_column_vector_is_false`, `observed_scalar_is_true`, `observed_matrix_is_false` | PASS |
| iscolumn.signature-primary, output-class, logical-column-test | FR-016-02 | `iscolumn::tests::observed_column_vector_is_true`, `observed_row_vector_is_false`, `observed_scalar_is_true`, `observed_matrix_is_false` | PASS |
| issparse.signature-primary, output-class, logical-sparse-test | FR-016-03 | `issparse::tests::observed_sparse_matrix_is_true`, `observed_full_matrix_is_false`, `observed_empty_sparse_is_true` | PASS |
| iscellstr.signature-primary, output-class, logical-cellstr-test | FR-016-04 | `iscellstr::tests::observed_cell_of_char_vectors_is_true`, `observed_mixed_cell_is_false`, `observed_plain_char_vector_is_false`, `observed_empty_cell_is_true` | PASS |
| istable.signature-primary, output-class, logical-table-test (false cases `numeric`, `cell`) | FR-016-05 | `istable::tests::observed_numeric_matrix_is_false`, `observed_cell_is_false` | PASS |
| istable.logical-table-test (true case `table`) | FR-016-05 | none — UNREACHABLE (no table type; Blocker B-010-1); documented note in `istable.rs` and `spec.md` | N/A (documented note) |
| unresolved choices | FR-016-06 | `isrow::…::unresolved_choice_{empty_shapes_follow_size_rule,nd_array_is_false,gpu_tensor_uses_handle_shape}`, `iscolumn::…::unresolved_choice_{empty_shapes_follow_size_rule,nd_array_is_false,gpu_tensor_uses_handle_shape}`, `issparse::…::unresolved_choice_{non_sparse_kinds_are_false,gpu_tensor_is_false_without_gather}`, `iscellstr::…::unresolved_choice_{string_scalar_elements_count_as_text,string_array_input_is_false,char_matrix_element_follows_kind,non_char_kinds_inside_cell_are_false}`, `istable::…::unresolved_choice_{every_constructible_kind_is_false,gpu_tensor_is_false_without_gather}` | PASS (non-normative) |

17 of the 18 observed cases across the five exports reproduce exactly
(value, class logical via `Value::Bool` ≡ 1×1 logical, per RunMat's value
model); the 18th (`istable` case `table`) is unreachable — see above.

## Deviations and notes

- `istable` is implemented always-false per the gate-ratified decision
  (RunMat has no table type; Blocker B-010-1 in
  `specs/010-table-conversion/spec.md`). This is correct for every
  constructible input; the observed true-case is a documented note citing
  istable.logical-table-test, and the feature returns to planning when a
  table type lands.
- `isrow`/`iscolumn` obtain shape via
  `builtins::common::shape::value_dimensions`, which answers GPU handles
  from shape metadata without gathering whenever the provider supplies it
  (verified by the `unresolved_choice_gpu_tensor_uses_handle_shape` tests);
  `issparse`/`iscellstr`/`istable` answer from the value kind with no
  provider call at all.
- `iscellstr` accepts `Value::String` elements (RunMat's internal
  string-scalar text representation) alongside `Value::CharArray` —
  documented independent choice on an unobserved input class
  (iscellstr.q-strings); char-matrix elements pass the kind-level test
  (also unobserved, documented choice).
- Registration is via the `#[runtime_builtin]` inventory macro in
  `logical/tests/mod.rs` (compile-time, same mechanism as all existing
  builtins); no pre-existing file other than that `mod.rs` was modified.
- Tests were executed while sibling feature agents worked concurrently in
  the same tree; the reported runs are the actual final outputs.

## Unresolved (returned to specification side)

`isrow.q-empty`, `iscolumn.q-empty`, `issparse.q-output`,
`iscellstr.q-strings`, `istable.q-timetable`.
