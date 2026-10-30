# Implementation Plan — 035-outerjoin

## Constitution Check

- I Specification-first: FR-035-01..12 cite the 16 approved claims.
  FR-035-13..22 are tagged unresolved or documented choice. PASS.
- II Clean-room: inputs are the imported 0.3.0 export plus RunMat code only.
  MATLAB not invoked. PASS.
- III Independence: a textbook equi-join (hash/ordered-map lookup plus a
  stable sort) over upstream's table model. It does not imitate any MATLAB
  internals. PASS.
- IV Provenance: `provenance.md` is complete, with the full pin and the
  freshness check. PASS.
- V Test-first: all tests are written and run (they fail with
  `RunMat:UndefinedFunction`) before the implementation. PASS.
- VII Minimal scope: one builtin. One new file plus one `mod` line in the
  hand-written `table/builtins.rs`. No upstream refactoring. The parallel
  F34 file `variable_editing.rs` is not touched. PASS.
- IX Compatibility: lives under `builtins/table/builtins/` and is
  registered via `#[runtime_builtin]` with a descriptor and `accel = "cpu"`.
  It reuses `select_rows`, `table_from_columns` and the `crate::callsite`
  API. Integer-capability metadata is deliberately omitted, as in F34. PASS.

## Affected files

| Item | Path |
|---|---|
| Builtin and tests (new) | `crates/runmat-runtime/src/builtins/table/builtins/joins.rs` |
| Registration | `mod joins;` in `crates/runmat-runtime/src/builtins/table/builtins.rs` |

## Design

1. Gather both inputs. Each must be a `table` (`InvalidInput`); a timetable
   gives `UnsupportedOption`. More than one requested output
   (`crate::output_count::current_output_count() > 1`) gives
   `UnsupportedOption`.
2. Parse the name-value pairs into `Keys`, `LeftKeys`, `RightKeys`,
   `MergeKeys` and `Type` (FR-035-22). `LeftVariables`/`RightVariables`
   give `UnsupportedOption`.
3. Resolve key pairs `(li, ri)`: inference (common names in left order,
   else `CantInferKey`), `Keys` (names or indices; `ConflictingInputs` with
   Left/RightKeys), or `LeftKeys`/`RightKeys`.
4. Read the variables, normalising scalar values (`Num`, `Bool`, `Int`,
   `String`) to one-element arrays. Convert each key column to atoms
   (`Float` / `Int(i128)` / `Text`), rejecting unsupported or mismatched
   classes, NaN and `<missing>` (FR-035-15/16).
5. Join: an ordered map from right key tuple to right rows (input order).
   Reject keys duplicated on both sides. For each left row in order, emit
   one `(Some(i), Some(j))` per match; if it has no match and the type keeps
   left rows, emit `(Some(i), None)`. Then emit unmatched right rows when
   the type keeps them. Stable-sort by key tuple (FR-035-05/06, U-035-1).
6. Columns: merge keys when `MergeKeys` is true, or for `'inner'` (FR-035-11;
   differently named keys without MergeKeys give `UnsupportedOption`).
   Order: left variables (merged key at the left key's position), then
   right variables minus merged right keys.
7. Names: a name appearing on both sides gets `_<left>` / `_<right>`, read
   from the call site (FR-035-17) or `_left`/`_right` otherwise. Any residual
   collision gets `_2`, `_3`, ….
8. Values: `select_rows` when a column needs no fill. Otherwise a typed
   gather with fill per FR-035-10/19 (numeric storage per class, logical,
   string, one-column cellstr); any other class gives `UnsupportedOption`.
   The merged key picks the left row if present, else the right row.
9. Build the result with `table_from_columns` (default properties, U-035-6).

## Test plan (inline `#[cfg(test)] mod table_outerjoin_tests`)

- Observed: one test per observation case or case group, 25 cases in all.
  Tests that depend on workspace names call through a recorded call site
  (`crate::callsite::push_callsite` plus a source-catalog entry), the same
  way the interpreter does.
- Unresolved: `unresolved_*` for each q-id.
- Documented choices: `documented_choice_*` for U-035-1..9.
- Validation: `rejects_*`, plus the empty inner join.

## Risks & rollback

- If the call site is stale (a builtin calling `outerjoin` internally),
  the wrong names could be picked. The argument-count guard (FR-035-17 b)
  limits this; the fallback is the observed `_left`/`_right`.
- Rollback: delete `joins.rs` and its `mod` line.

## Verification commands

`cargo test -p runmat-runtime --lib -- table_outerjoin` ·
`cargo test -p runmat-runtime --lib -- builtins::table` ·
`cargo clippy -p runmat-runtime --lib --tests --message-format short 2>&1 | grep joins` ·
`rustfmt --edition 2021 --check <files>`
