# Tasks — 036-graph-structure (spec 0.3.0)

Order is normative (test-first). Every task cites requirements. Only work
actually done is ticked.

- [x] **T0** Freshness check of the source repository for both specs
  (fetch; diff `8614c5b` vs `HEAD` and `origin/main`; status). [provenance]
- [x] **T1** Inspect the upstream graph subsystem (`GraphData`, `Edge`,
  `graph_data`, edge storage order, class constants, attribute style) and
  `SparseTensor::new`. [FR-036-01..11] (no code changes)
- [x] **T2** Rewrite the observed normative tests to 0.3.0 (incidence and
  isisomorphic case groups, error identifiers). [FR-036-01..07; SC-036-1]
- [x] **T3** Author unresolved/documented-choice tests (named nodes, extra
  args, arity, options, second output, search cases) and the ignored
  D-036-1 deviation test. [FR-036-08..11]
- [x] **T4** Run the test filter against the earlier implementation and
  record the failures before changing it (10 passed, 7 failed, 1 ignored).
- [x] **T5** Update `structure.rs` to 0.3.0: digraph support, `SelfLoops`,
  `UndefinedFunction`, `ClassMismatch`, second-output rejection.
  [FR-036-01..09]
- [x] **T6** Register: `mod structure;` in `graph/domain/mod.rs`. [SC-036-2]
- [x] **T7** Run the verification commands, including the upstream graph
  suite and clippy; record output in `validation.md`. [SC-036-2, SC-036-4]
- [x] **T8** Traceability table (claim → FR → test → result) in
  `validation.md`. [SC-036-3; Principle VI]
