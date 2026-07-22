# Validation Report — 005-struct-cell-access — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (1m 35s)
- `cargo fmt --all -- --check` — clean
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::structs::core::fields builtins::cells::core::num2cell` — **17 passed, 0 failed** (6531 filtered out)
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::cells::type_resolvers::tests::num2cell_type_is_cell` — **1 passed, 0 failed**

## Traceability (claim → FR → test → result)

| Claim | FR | Test | Result |
|---|---|---|---|
| fields.signature-primary, output-class, cell-fieldnames | FR-005-01 | `fields::tests::observed_two_field_struct_yields_2x1_cell`, `observed_empty_struct_yields_0x1_cell` | PASS |
| (summary) fields ≡ fieldnames legacy equivalent | FR-005-02 | delegation body (`call_builtin_async("fieldnames", …)`); equivalence exercised by `fields::tests::unresolved_choice_struct_array_matches_fieldnames` | PASS (summary-derived) |
| unresolved choices (fields) | FR-005-03 | `fields::tests::unresolved_choice_non_struct_input_errors`, `unresolved_choice_struct_array_matches_fieldnames` | PASS (non-normative) |
| num2cell.signature-primary, output-class, cell-grouping | FR-005-04 | `num2cell::tests::observed_row_vector_yields_1x3_cell`, `observed_matrix_yields_2x2_cell`, `observed_scalar_yields_1x1_cell` | PASS |
| num2cell.cell-grouping (dim form, outer size observed) | FR-005-05 | `num2cell::tests::observed_dim1_yields_1x2_cell` | PASS |
| (claim statement) dim groups elements — cell contents | FR-005-05 | `num2cell::tests::summary_derived_dim1_cells_are_columns` | PASS (summary-derived) |
| unresolved choices (num2cell) | FR-005-06 | `num2cell::tests::unresolved_choice_empty_matrix_yields_0x0_cell`, `unresolved_choice_dim_vector_groups_both_dimensions`, `unresolved_choice_char_row_yields_char_cells`, `unresolved_choice_invalid_dim_errors` | PASS (non-normative) |
| — (plumbing, no conformance claim) | — | `fields::tests::descriptor_signature_covers_fields_form`, `num2cell::tests::descriptor_signatures_cover_num2cell_forms`, `too_many_arguments_error`, `gpu_input_gathers_to_host` | PASS |

All 6 observed cases across the two exports reproduce exactly (class cell
and observed size; contents asserted only where an approved claim statement
covers them).

## Deviations and notes

- `fields` delegates to the registered `fieldnames` builtin through
  `call_builtin_async` (the `bounds` → `min`/`max` pattern), so errors for
  non-struct inputs carry the delegate's message/identifier
  (`RunMat:fieldnames:InvalidTarget`) — documented independent choice; error
  inputs are unobserved in the approved spec.
- The `fields` observation records only the cell class/size, not the name
  contents or ordering; RunMat returns sorted names (inherited from
  `fieldnames`) — documented independent choice.
- `num2cell` cell data is assembled row-major (RunMat `CellArray` storage
  order) while tensor blocks are extracted column-major (RunMat tensor
  order); the matrix test pins per-position contents via `CellArray::get`.
- Integer scalar inputs are converted through the shared f64 tensor helper
  (mirroring `mat2cell`); integer-class preservation is unobserved and left
  to the specification side.
- The generated WASM registry (`generated_wasm_registry.rs`) was not
  regenerated, matching the precedent of features 002–004 on this branch;
  native registration is via the `#[runtime_builtin]` inventory macro.

## Unresolved (returned to specification side)

`fields.q-alias` (exact fieldnames equivalence, deprecation status, name
ordering), `num2cell.q-shape` (cell contents/shape for each form, especially
with dim; empty, char, integer-class and error behaviour).
