# Tasks — 035-outerjoin

Order is normative (test-first). Every task cites requirements.

- [x] **T0** Spec freshness check against `matlab-interface-spec` (pin
  `5d595da`), at start: no newer or in-progress change to
  `specifications/outerjoin`. Imported files byte-identical. [Principle IV]
- [x] **T1** Inspect the substrate: the table object model, `select_rows`,
  `table_from_columns`, numeric/logical/string/cell storage, the
  `crate::callsite` record used by `display` and `inputname`, and
  `crate::output_count`. No code changes. [FR-035-01..22]
- [x] **T2** Author observed normative tests for all 25 observation cases.
  Cases that depend on workspace names run through a recorded call site.
  [FR-035-01..12; SC-035-1]
- [x] **T3** Author the unresolved-disposition tests
  (`outerjoin.q-index-outputs`, `q-select-variables`, `q-many-to-many`,
  `q-key-types`, `q-name-suffix-context`). [FR-035-13..17]
- [x] **T4** Author the documented-choice and validation tests (U-035-1..9,
  non-table and timetable inputs, malformed options, empty inner join). Run
  `cargo test -p runmat-runtime --lib -- table_outerjoin` and record that
  it fails because the builtin is absent. [FR-035-18..22]
- [x] **T5** Implement `joins.rs`: descriptor (5 signatures, 7 error
  identifiers), option parsing, key resolution, key atoms, join and sort,
  column layout, naming with call-site suffixes, and fill assembly.
  Integer-capability metadata is deliberately omitted. [FR-035-01..22]
- [x] **T6** Register: `mod joins;` in `table/builtins.rs` (the inventory
  macro registers the builtin). [SC-035-2]
- [x] **T7** Run the verification commands (feature tests, `builtins::table`
  suite, clippy, rustfmt check) and record the output in `validation.md`.
  Repeat the freshness check at finish. [SC-035-1..3]
- [x] **T8** Write the traceability table (claim → FR → test → result) in
  `validation.md`. [SC-035-3; Principle VI]
