# Implementation Plan — 046-isprop

## Constitution Check

- I Specification-first: FR-046-01..06 cite the 7 approved claims.
  FR-046-07..12 are tagged as documented choices or unresolved. PASS.
- II Clean-room: the only inputs are the imported export and RunMat code.
  MATLAB was not invoked. PASS.
- III Independence: the feature reuses RunMat's own handle resolver,
  property getter and class registry. PASS.
- IV Provenance: `provenance.md` is pinned to `c65e238` and was
  freshness-checked. The 7 claim ids were verified. PASS.
- V Test-first: tests are written before the body, and the failing run is
  recorded in `validation.md`. PASS.
- VII Minimal scope: one new file, plus one `pub mod isprop;` line in
  `introspection/mod.rs`. **That parent is generated ("Do not edit")**, so
  the line follows the `strmatch` and `findstr` precedent and must be listed
  in the maintainer README of generated-parent edits. PASS (flagged).
- IX Compatibility:
  - Registration uses `#[runtime_builtin]` with a descriptor; the builtin
    runs on the host.
  - The input is gathered with `gather_if_needed_async`.
  - No integer-capability metadata is added. PASS.

## Affected files

| Item | Path |
|---|---|
| Builtin (new) | `crates/runmat-runtime/src/builtins/introspection/isprop.rs` |
| Registration | `pub mod isprop;` in `crates/runmat-runtime/src/builtins/introspection/mod.rs` (generated parent; flagged) |

## Design

1. Arity:
   - check 2 arguments (`NotEnoughInputs` / `TooManyInputs`);
   - `name` must be a char row or string scalar, otherwise
     `UnsupportedOption`;
   - gather `obj`.
2. Dispatch on `obj`:
   - Table or timetable (`crate::builtins::table::is_table_value`) →
     `false`.
   - `Value::Object` or `Value::HandleObject` → `class_answer(class, name)`:
     - exact match among the public property names (the registry walk,
       shared with `properties`) → true;
     - a match only when case is ignored → `UnsupportedOption`;
     - otherwise → false.
   - `Value::ObjectArray` → a same-shape logical holding the class answer.
   - `Value::MException` → `class_answer` over `identifier`, `message` and
     `stack`.
   - `Value::Num` or an F64 `Value::Tensor` whose elements all resolve as
     plot handles:
     - each element gives `get_properties(handle, Some(name)).is_ok()`;
     - a scalar gives `Bool`, an array gives a `LogicalArray` of the same
       shape;
     - if not every element resolves, or the array is empty → `false`.
   - `Value::ClassRef` → `UnsupportedOption`.
   - Everything else → `false`.

## Test plan

Tests are an inline `#[cfg(test)] mod isprop_tests`, calling through
`crate::call_builtin_async`. Graphics tests hold the plot test lock and use
`figure_builtin` / `subplot_builtin`.

- Each observed case has one `observed_*` test (11).
- `documented_choice_*` and `unresolved_*` tests cover FR-046-07..12.

## Risks & rollback

- If the module composition is regenerated, `pub mod isprop;` is dropped
  unless the generator learns about it.
- Graphics property support is only as complete as RunMat's `get`.
- Rollback: delete `isprop.rs` and its `mod` line.

## Verification commands

- `cargo test -p runmat-runtime --lib -- isprop_tests`
- `cargo test -p runmat-runtime --lib -- introspection`
- clippy, filtered to `isprop.rs`
- `rustfmt --edition 2021 --check`
