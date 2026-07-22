# Tasks — 002-type-predicates

Order is normative (test-first). Every task cites requirements.

- [ ] **T1** Inspect `Value` variants for cell/struct(array) representation;
  confirm predicate mapping. [FR-002-01, FR-002-02] (no code changes)
- [ ] **T2** Author failing tests for `iscell`/`isstruct` observed cases in
  new files' test modules (tests reference not-yet-written fns — this is the
  red step; compile gated by T4). [FR-002-01, FR-002-02; SC-002-1]
- [ ] **T3** Author failing tests for `isfile`: missing→false (normative),
  existing→true (`summary_derived`), folder→false (`unresolved_choice`).
  [FR-002-03, FR-002-04, FR-002-05]
- [ ] **T4** Implement `iscell.rs` + registration (descriptor, macro,
  mod entry). [FR-002-01]
- [ ] **T5** Implement `isstruct.rs` + registration. [FR-002-02]
- [ ] **T6** Implement `isfile.rs` + registration via fs provider.
  [FR-002-03..05]
- [ ] **T7** Run verification commands; record output in `validation.md`.
  [SC-002-1..3]
- [ ] **T8** Traceability table in `validation.md` (claim → FR → test →
  result). [SC-002-3; Principle VI]
