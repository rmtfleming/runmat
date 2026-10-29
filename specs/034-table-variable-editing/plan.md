# Implementation Plan — 034-table-variable-editing

## Constitution Check

- I Specification-first: FR-034-01..03 cite the eight approved claims;
  FR-034-04..10 are tagged unresolved or documented choice. PASS.
- II Clean-room: inputs are the imported exports plus RunMat code only;
  MATLAB not invoked. PASS.
- III Independence: variable editing is a reorder/insert/delete over
  upstream's ordered variable struct, derived from RunMat's own table model.
  PASS.
- IV Provenance: `provenance.md` complete, with pins and the 2026-10-29
  re-verification. PASS.
- V Test-first: tests T2–T4 are written and run (and fail to resolve the
  builtins) before T5. PASS.
- VII Minimal scope: one tightly coupled family; one new file plus one `mod`
  line in a hand-written (non-generated) parent; no refactoring of upstream
  table code. PASS.
- IX Compatibility: lives under `builtins/table/builtins/`, registered via
  `#[runtime_builtin]` with a descriptor and `accel = "cpu"`, using upstream's
  table helpers. Integer-capability metadata is deliberately omitted: declaring
  it ties a builtin to the generated `docs/builtins/meta.json`
  (`generated_builtin_catalog_matches_integer_metadata`), which this fork
  cannot regenerate. PASS.

## Affected files

| Item | Path |
|---|---|
| Builtins (new) | `crates/runmat-runtime/src/builtins/table/builtins/variable_editing.rs` |
| Registration | `mod variable_editing;` in `crates/runmat-runtime/src/builtins/table/builtins.rs` (hand-written, not a generated parent) |

Exemplars: `table/builtins/conversions.rs` (descriptor, gather pattern), `table/object/properties.rs`
(`table_public_properties`, `sync_table_properties`, units/descriptions
handling), `table/names.rs` (default names).

## Design

- Shared core `rebuild(T, slots) -> table` (plus `table_input` for step 1):
  1. `gather_if_needed_async` the inputs; require `Value::Object` of class
     `table` (`TABLE_CLASS`). A timetable gives `UnsupportedOption` (U-034-3),
     anything else `InvalidInput`.
  2. Read `names = table_variable_names_from_object`,
     `vars = table_variables`, `props = table_public_properties`.
  3. Compute the new order as `Vec<Source>` with
     `Source::{Existing(old_idx), New(value)}`.
  4. Build the new ordered `StructValue`. Permute `VariableUnits` and
     `VariableDescriptions` by the same index map (empty string for new);
     set `VariableNames`. Write back into a clone of the source object via
     `TABLE_VARIABLES_FIELD` and `sync_table_properties`. Row names,
     Description, UserData and DimensionNames are untouched.
- `addvars(T, var)`: `rest.is_empty()` else `UnsupportedOption`;
  `value_row_count(var) == table_height(T)` else `HeightMismatch`; name
  `Var{width+1}` made unique against existing names (existing names never
  change).
- `removevars(T, vars)`: one text name (char row or string scalar);
  numeric/logical/cell/multi-element string → `UnsupportedOption`; unknown →
  `UnknownVariable`.
- `movevars(T, vars, key, loc)`: exactly 4 args; `key` text equal to
  `"before"` ignoring case, `"after"` → `UnsupportedOption`, other →
  `InvalidArgument`; `vars` single text else `UnsupportedOption`; `loc` text
  else (numeric) `UnsupportedOption`; unknown names → `UnknownVariable`;
  `vars == loc` → unchanged.
- Errors: per-builtin `BuiltinErrorDescriptor` sets with identifiers
  `RunMat:<fn>:{InvalidInput,InvalidArgument,UnsupportedOption,UnknownVariable,HeightMismatch}`.

## Test plan (inline `#[cfg(test)] mod table_variable_editing_tests`)

Observed (normative, cite claims): `observed_addvars_append_matches_export`,
`observed_removevars_by_name_matches_export`,
`observed_movevars_before_matches_export` (size and `c,a,b` order).
Unresolved (`unresolved_choice_*`): every disposition in spec.md US4, plus
default naming, metadata preservation, and moving a variable before itself.
Validation (`rejects_*`): non-table input, timetable, unknown names, height
mismatch, bad keyword. Plus `accepts_string_scalar_names`.

## Risks & rollback

- Upstream table internals (`TABLE_VARIABLES_FIELD`, Properties) are
  `pub(in crate::builtins::table)`. The new file lives inside that module,
  so no visibility changes are needed.
- Rollback: delete `variable_editing.rs` and its `mod` line.

## Verification commands

`cargo check -p runmat-runtime` ·
`cargo test -p runmat-runtime --lib -- table_variable_editing` ·
`cargo test -p runmat-runtime --lib -- builtins::table` (no regressions) ·
`cargo fmt --all -- --check` · `cargo clippy -p runmat-runtime --lib --tests`
