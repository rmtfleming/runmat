# Tasks — 044-table-stack-innerjoin

Order is normative (test-first). Every task cites requirements.

- [x] **T0** Spec freshness check against `matlab-interface-spec` (pin
  `c65e238`), at start: no newer or in-progress change to
  `specifications/{stack,innerjoin}`. [Principle IV]
- [x] **T1** Inspect the substrate: table object model, `select_rows`,
  `table_from_columns`, the categorical object (`Codes`/`Categories`),
  `crate::callsite`, `crate::output_count`, and F35 `joins.rs`. No code
  changes. [FR-044-01..36]
- [x] **T2** Author observed normative tests for all 9 `stack` and 11
  `innerjoin` cases. [FR-044-01..18; SC-044-1]
- [x] **T3** Author unresolved-disposition tests (`stack.q-second-output`,
  `stack.q-groups`, `innerjoin.q-index-outputs`, `innerjoin.q-many-to-many`,
  `innerjoin.q-name-suffix-context`). [FR-044-20..24, FR-044-34]
- [x] **T4** Author documented-choice and validation tests (U-044-1..11).
  Add the `mod` lines with stub files containing only tests, run
  `cargo test -p runmat-runtime --lib -- table_stack_tests table_innerjoin_tests`
  and record the failing run. [FR-044-25..36]
- [x] **T5** Implement `stack.rs` (descriptor, vars/options parsing,
  interleave, categorical indicator). [FR-044-01..07, 20..22, 25..30]
- [x] **T6** Implement `innerjoin.rs` (descriptor, options, keys, join,
  layout, call-site suffixes). [FR-044-10..18, 22..24, 31..36]
- [x] **T7** Run feature tests, the `builtins::table` suite, clippy and
  rustfmt check; repeat the freshness check; record in `validation.md`.
  [SC-044-1..2]
- [x] **T8** Traceability table (claim → FR → test → result) and deviations
  in `validation.md`. [SC-044-3; Principle VI]
