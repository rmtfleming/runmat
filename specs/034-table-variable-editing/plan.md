# Implementation Plan — 034-table-variable-editing (widened to 0.3.0)

## Constitution Check

- I Specification-first: FR-034-01..09 cite approved 0.3.0 claims. FR-034-10
  is a recorded deviation, and FR-034-11..16 are tagged identifier mapping,
  unresolved or documented choice. PASS.
- II Clean-room: inputs are the imported 0.3.0 exports plus RunMat code only;
  MATLAB not invoked. PASS.
- III Independence: insert/delete/reorder over upstream's ordered variable
  struct. Caller names come from RunMat's own builtin callsite record. PASS.
- IV Provenance: `provenance.md` pins `5d595da…`, lists every relied-upon
  claim id (verified against the imported `provenance.yaml`), and records
  the freshness checks. PASS.
- V Test-first: the widened tests are written and run against the 0.2.0-era
  implementation, and their failures are recorded before the implementation
  changes (validation.md). PASS.
- VII Minimal scope: one existing file (`variable_editing.rs`) plus this
  feature's docs. No upstream table code changes. PASS.
- IX Compatibility: same registration (`#[runtime_builtin]`, descriptors,
  `accel = "cpu"`), upstream table helpers, and the existing
  `crate::callsite` API. PASS.

## Affected files

| Item | Path |
|---|---|
| Builtins (edited) | `crates/runmat-runtime/src/builtins/table/builtins/variable_editing.rs` |
| Feature docs | `specs/034-table-variable-editing/*.md` |

Registration (`mod variable_editing;` in `table/builtins.rs`) is unchanged.

## Design

- **Errors.** A `Kind` enum and one descriptor set per builtin (macro
  `error_set!`). Each builtin lists only the kinds it can raise.
  `Ctx::error(kind, detail)` looks the identifier up in the descriptor set,
  so descriptors stay the single source of truth. Table-level identifiers
  follow FR-034-11 (`RunMat:table:…`, `RunMat:badsubscript`).
- **Selections** (`select_variables`, shared by removevars/movevars):
  logical mask (length must equal width, else `UnsupportedOption`), cell of
  names, string array, single name, or numeric indices
  (`index_position`: non-positive or non-integer → `BadSubscript`,
  > width → `VarIndexOutOfRange`). The result is 0-based positions in listed
  order (table order for masks), with repeats removed.
- **Options.** `option_start` finds the first recognised keyword after the
  mandatory first data argument. Everything from there on must be
  name-value pairs (`parse_options`): odd count or unknown name →
  `InvalidArgument`, repeat → `UnsupportedOption`. `placement` raises
  `BeforeAndAfter` and resolves the location against the input table
  (`location_position`: name, or one number; `'Before'` width+1 →
  `UnsupportedOption`).
- **addvars.** Gather the data arguments. Every one must match the table
  height (the first argument's height when the table has no variables).
  Insertion index: end / anchor / anchor+1. Names: `NewVariableNames`
  (count, validity and duplicate checks), else caller name (`caller_names`
  reads `crate::callsite::current_callsite()`/`arg_text`, trusted only when
  span count = argument count and no earlier argument may expand), else
  `Var<insert_at+offset+1>`. Defaults are made unique with
  `make_unique_names`. The new slots are spliced in and `rebuild`.
- **removevars.** Keep the unselected positions. If that leaves zero
  variables and height > 0, raise `UnsupportedOption` (D-034-1).
- **movevars.** `vars` plus optional Before/After pair. Self-location with
  one variable → unchanged. A location inside a multi-variable selection →
  `UnsupportedOption`. Otherwise remove the moved positions, find the
  anchor in the remainder, and splice the block in listed order.
- `rebuild` (unchanged) carries units and descriptions with their
  variables and keeps the other metadata.

## Test plan (inline `mod table_variable_editing_tests`)

- `observed_*` (normative, cite claim ids and case ids): one test per claim
  group: addvars append / append-full / before-after / new names / multiple
  / data-preserved / name-from-input / errors; removevars by-name /
  index+mask / multiple names / empty selection / errors; movevars before /
  after+numeric location / selectors+order / self / default end / contents /
  errors.
- `unresolved_*`: before-past-end, mask length, location-in-vars.
- `unresolved_choice_*`: default-name collision, input-name scope (callsite
  simulated with `source_context::replace_source_catalog` +
  `callsite::push_callsite`, as `display`'s tests do), metadata.
- `deviation_*`: remove-all with rows.
- `rejects_*` / `accepts_*`: non-table, timetable, malformed options, string
  scalars.

## Risks & rollback

- Caller-name lookup depends on the VM pushing a builtin callsite. Without
  one, names fall back to `Var<k>`. The span-count check guards against
  stale or foreign callsites, but a nested Rust-level call with the same
  argument count could in principle read an outer callsite (the same
  exposure `display` has).
- Rollback: restore `variable_editing.rs` from commit `69a30fe2c`.

## Verification commands

`cargo test -p runmat-runtime --lib -- table_variable_editing` ·
`cargo test -p runmat-runtime --lib -- builtins::table` ·
`rustfmt --edition 2021 --check <file>` ·
`cargo clippy -p runmat-runtime --lib --tests --message-format short | grep variable_editing`
