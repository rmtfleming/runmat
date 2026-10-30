# Implementation Plan — 044-table-stack-innerjoin

## Constitution Check

- I Specification-first: FR-044-01..18 cite the 20 approved claims.
  FR-044-20..36 are tagged unresolved or documented choice. PASS.
- II Clean-room: inputs are the imported 0.2.0 exports plus RunMat code only.
  MATLAB not invoked. PASS.
- III Independence: an unpivot and a textbook equi-join over upstream's
  table model. PASS.
- IV Provenance: `provenance.md` with the full pin and freshness checks. PASS.
- V Test-first: all tests written and run (failing with
  `RunMat:UndefinedFunction`) before the implementation. PASS.
- VII Minimal scope: two new files plus two `mod` lines in the hand-written
  `table/builtins.rs`. `joins.rs` and other agents' files are not touched;
  the join helpers needed are copied (noted in `provenance.md`). PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptors and
  `accel = "cpu"`, reusing `select_rows`, `table_from_columns`,
  `crate::callsite` and `crate::output_count`. Integer-capability metadata
  deliberately omitted (as F34/F35). PASS.

## Affected files

| Item | Path |
|---|---|
| `stack` builtin and tests (new) | `crates/runmat-runtime/src/builtins/table/builtins/stack.rs` |
| `innerjoin` builtin and tests (new) | `crates/runmat-runtime/src/builtins/table/builtins/innerjoin.rs` |
| Registration | `mod innerjoin;` and `mod stack;` in `crates/runmat-runtime/src/builtins/table/builtins.rs` |

## Design — `stack`

1. Gather `T`; it must be a table (`InvalidInput`; timetable and row names
   `UnsupportedOption`). More than one output → `UnsupportedOption`.
2. Resolve `vars` (names, string array, cellstr, indices) to positions in
   listing order. A cell element that is itself a cell → `UnsupportedOption`
   (groups). Unknown name → `RunMat:table:UnrecognizedVarName`.
3. Parse options `NewDataVariableName`, `IndexVariableName`,
   `ConstantVariables` (case-insensitive).
4. Carried variables: all non-stacked variables in table order, or the
   `ConstantVariables` list in table order.
5. Check the stacked classes: double + cell → `InterleavingDataVarsFailed`;
   any other mix, column mismatch or unsupported class → `UnsupportedOption`.
6. Interleave: output row `i·k + v` = row `i` of stacked variable `v`
   (column-major per storage class). Carried variables are
   `select_rows(col, [0,0,…,1,1,…])` (each row repeated `k` times).
7. Indicator: a categorical object (`Codes` = `v+1` per row, `Categories` =
   stacked names in listing order, non-ordinal).
8. `table_from_columns(carried ++ [indicator, data])`; name collisions →
   `InvalidArgument`.

## Design — `innerjoin`

1. Gather both inputs (tables only). More than one output →
   `UnsupportedOption`.
2. Parse `Keys`, `LeftKeys`, `RightKeys`, `LeftVariables`; `RightVariables`
   → `UnsupportedOption`.
3. Key pairs: inference (common names in left order; none → `CantInferKey`),
   `Keys`, or `LeftKeys`/`RightKeys` (copied from F35).
4. Key atoms per row (copied from F35), ordered map right key → rows;
   many-to-many → `UnsupportedOption`. Matched `(i, j)` pairs, stable sort by
   key tuple (left order for equal keys).
5. Layout: selected left keys in key order, other selected left variables
   in table order, right non-key variables in table order.
6. Clashing names (left-side vs right-side) get `_<caller name>` suffixes via
   `crate::callsite`, else `_left`/`_right`; leftover collisions `_2`, ….
7. Values via `select_rows` on each source column.

## Tests

Module names `table_stack_tests` and `table_innerjoin_tests`; categories
`observed_*`, `unresolved_*`, `documented_choice_*`, `rejects_*`. Call-site
dependent cases use the F35 `call_at` helper (copied). Output-count tests
call the builtin directly under `crate::output_count::push_output_count`.
