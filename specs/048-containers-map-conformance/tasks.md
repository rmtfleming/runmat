# Tasks — 048-containers-map-conformance (specs 0.2.0)

Order is normative (test-first). Every task cites requirements. Only work
actually done is ticked.

- [x] **T0** Freshness check of the source repository for the three specs
  (fetch; log `8f72edb..HEAD` and `..origin/main`; status). [provenance]
- [x] **T1** Inspect the upstream map code (`MapStore`, `normalize_key`,
  `collect_key_spec`, class registration, dispatcher instance-method route,
  warning emission). [FR-048-01..12] (no code changes)
- [x] **T2** Write `conformance_tests.rs` with the observed tests for the
  three functions and the `unresolved_choice_*` tests; register it in
  `containers/map/mod.rs`. [FR-048-01..13; SC-048-1]
- [x] **T3** Run the test filter against the unchanged upstream code; record
  the failing run in `validation.md`.
- [x] **T4** Minimal fix: `isKey` class mismatch → false; cell query in
  column-major order. [FR-048-06, FR-048-08, U-048-3]
- [x] **T5** Minimal fix: `remove` warns `RunMat:Containers:Map:NoKeyToRemove`
  instead of erroring; update upstream test
  `remove_missing_key_returns_error`. [FR-048-12, U-048-2]
- [x] **T6** Fix any further divergence shown by T3 (member form `M.keys`).
  [FR-048-04]
- [x] **T7** Run the verification commands (feature tests, containers suite,
  rustfmt, clippy); record output in `validation.md`. [SC-048-2]
- [x] **T8** Traceability table, behaviour-change list and final freshness
  check in `validation.md`. [SC-048-3; Principle VI]
