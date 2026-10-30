# Tasks — 042-sparse-orderings (`amd`, `etree`, `symbfact` 0.2.0)

Order is normative (test-first). Every task cites requirements. Ticks mark
work actually done.

- [x] **T0** Freshness check of the source spec repository (read-only) at
  the start of work. [Principle IV]
- [x] **T1** B1: `spec.md`, `provenance.md` (pin `c65e238`; 25 claim ids and
  7 question ids verified against the import). [FR-042-01..22]
- [x] **T2** B2: `plan.md`, this file. [all]
- [x] **T3** Write the `observed_*` tests (8 `amd`, 14 `etree`, 9 `symbfact`
  cases). [FR-042-01..14; SC-042-1]
- [x] **T4** Write the `documented_choice_*` / `unresolved_*` tests. Register
  the test-only modules, run the filter, and record the failing run
  (`UndefinedFunction`) in validation.md. [FR-042-15..22]
- [x] **T5** Implement `symbolic.rs` (pattern reader, upper columns, trees,
  postorder, output helpers). [FR-042-06..10, 17, 19, 20]
- [x] **T6** Implement `amd.rs` (descriptor, minimum degree). [FR-042-01..05,
  15, 16, 21]
- [x] **T7** Implement `etree.rs`. [FR-042-06..10, 17, 18, 21]
- [x] **T8** Implement `symbfact.rs`. [FR-042-11..14, 18, 19, 21]
- [x] **T9** Run the verification commands (feature tests, `math::sparse`
  and `factor::svd` regressions, clippy, rustfmt) and the end-of-work
  freshness check; record them in validation.md. [SC-042-1..3]
- [x] **T10** Traceability table (claim → FR → test → result) in
  validation.md. [SC-042-3; Principle VI]
