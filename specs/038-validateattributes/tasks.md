# Tasks — 038-validateattributes (spec 0.3.0)

Order is normative (test-first). Every task cites requirements. Only tasks
actually completed are ticked.

- [x] **T0** Freshness check of the source spec repo (read-only) and
  re-import verification; rewrite spec/provenance/plan for 0.3.0; verify
  all cited claim ids. [FR-038-01..08]
- [x] **T1** Inspect RunMat helpers: `class_name_for_value`,
  `value_dimensions`, `crate::warnings::emit`, `warning_store`,
  `output_count::current_output_count`, `gather_if_needed_async`.
  [FR-038-06, FR-038-08, FR-038-09, FR-038-16]
- [x] **T2** Author observed tests for all 58 cases (pass silently, mapped
  RunMat identifier, warning, `maxlhs`). [FR-038-01..08; SC-038-1]
- [x] **T3** Author documented-choice and unresolved-disposition tests.
  [FR-038-09..16]
- [x] **T4** Re-add `pub mod validateattributes;`, run the filter, record
  that it fails (`UndefinedFunction`) before T5. [SC-038-2]
- [x] **T5** Implement the builtin (descriptor, error set, parsing,
  checks, warning). [FR-038-01..16]
- [x] **T6** Run verification commands (tests, clippy, rustfmt on the
  file); record output in `validation.md`. [SC-038-1..3]
- [x] **T7** Traceability table (claim → FR → test → result) in
  `validation.md`. [SC-038-3]

Notes: T5 is complete except end-to-end `maxlhs` through the dispatcher
(validation.md D-1, upstream dispatcher retry). T6 clippy is clean for this
file; the crate-wide clippy exit is non-zero because of another agent's file.
