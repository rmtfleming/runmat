# Implementation Plan — 054-types-structs-text

## Constitution Check

- I Specification-first: FR-054-01..22 cite the 24 normative claims;
  FR-054-23..28 are tagged as documented choices or unresolved, and the two
  non-normative claims are used only as the basis of documented choices.
  PASS.
- II Clean-room: the only inputs are the imported exports and RunMat code.
  MATLAB was not invoked. PASS.
- III Independence: class-name test, `isnan` of stored serials, a
  field-major copy into a cell, and row stacking over RunMat's `char`.
  PASS.
- IV Provenance: `provenance.md` pins N3 to `1156c97` and N4 to `fc9f804`,
  freshness-checked; all 26 claim ids verified. PASS.
- V Test-first: tests are written before the bodies; the failing run is
  recorded in `validation.md`. PASS.
- VII Minimal scope: four new files, one `pub mod` line each in three
  **hand-written** parents (`logical/tests/mod.rs` ×2, `structs/core/mod.rs`,
  `strings/core/mod.rs`; none carries the "@generated" header). No generated
  parent is edited. PASS.
- IX Compatibility: `#[runtime_builtin]` with inline descriptors; results
  are `Value::Bool`, `Value::LogicalArray`, `Value::Cell`,
  `Value::CharArray`. No integer-capability metadata. PASS.

## Placement

| Builtin | File | Parent (hand-written) |
|---|---|---|
| `isfloat` | `crates/runmat-runtime/src/builtins/logical/tests/isfloat.rs` | `logical/tests/mod.rs` (next to `isnumeric`, `isreal`) |
| `isnat` | `crates/runmat-runtime/src/builtins/logical/tests/isnat.rs` | `logical/tests/mod.rs` (next to `isnan`; reads datetime storage through `crate::builtins::datetime`'s `pub(crate)` helpers, so the generated `datetime/mod.rs` is not touched) |
| `struct2cell` | `crates/runmat-runtime/src/builtins/structs/core/struct2cell.rs` | `structs/core/mod.rs` |
| `strvcat` | `crates/runmat-runtime/src/builtins/strings/core/strvcat.rs` | `strings/core/mod.rs` (next to `char`) |

## Design

### isfloat
1. Outputs ≤ 1; 0 inputs → `RunMat:minrhs`; >1 → `RunMat:maxrhs`.
2. `class_name_for_value(A)` ∈ {`double`, `single`} → `Value::Bool`.

### isnat
1. Outputs ≤ 1; exactly one input, else `RunMat:isnat:InvalidInput`.
2. Not `is_datetime_object` → `RunMat:isnat:UndefinedFunction`.
3. `serials_from_datetime_value` → tensor; map `is_nan`; 1×1 →
   `Value::Bool`, otherwise `LogicalArray` with the tensor's shape.

### struct2cell
1. Outputs ≤ 1; exactly one input, else `RunMat:struct2cell:InvalidInput`.
2. `Value::Struct(s)`: k fields, shape [1,1]; `Value::StructArray(a)`: field
   columns `a.field_values(name)` (column-major), shape `a.shape()`. Other
   → `RunMat:struct2cell:UndefinedFunction`.
3. Data (column-major): for element i, for field f → `values[f][i]`.
   Shape `[k] ++ shape`, trailing 1s dropped while more than 2 dims remain.
4. `CellArray::from_column_major`.

### strvcat
1. Outputs ≤ 1. Gather each input.
2. If any input is a cell and any is not → `RunMat:character:InvalidInput`.
3. Per input: struct / struct array → `RunMat:invalidConversion`; logical
   → `UnsupportedOption`; char, string, numeric → `char(input)`; cell →
   each element (column-major) that is char or string → `char(elem)`, any
   other element → `UnsupportedOption`; anything else → `UnsupportedOption`.
4. Each converted char matrix with no elements is skipped; otherwise all
   its rows are appended. Pad to the widest row. No rows → 0×0 char.

## Test plan

Inline `#[cfg(test)]` modules `isfloat_tests`, `isnat_tests`,
`struct2cell_tests` and `strvcat_tests`, calling through
`crate::call_builtin_async` (output-count tests call the builtin function
directly under `push_output_count`). Inputs such as `sparse(1)`,
`int8(1)`, `@sin` and `datetime(2020,1,2)` are built by calling RunMat's
own builtins where they exist, else as `Value`s directly.

- One `observed_*` test per observed case (50; the non-normative error
  cases are `documented_choice_*`).
- NaT values are built with `datetime_object_from_serial_tensor` and a NaN
  serial (D-054-6). `from-text-nat` calls RunMat's `datetime('NaT')`
  (D-054-7).

## Risks & rollback

- `strvcat` depends on `char`'s conversion rules; an upstream change shows
  up in the `observed_numeric` / `observed_string` tests.
- Rollback: delete the four files and their `pub mod` lines.

## Verification commands

- `cargo test -p runmat-runtime --lib -- isfloat_tests isnat_tests struct2cell_tests strvcat_tests`
- `cargo test -p runmat-runtime --lib -- builtins::logical::tests builtins::structs builtins::strings::core` (regressions)
- `rustfmt --edition 2021 --check <files>`
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`,
  filtered to the four files
