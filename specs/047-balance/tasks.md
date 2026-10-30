# Tasks — 047-balance (`balance` 0.2.0)

Order is normative (test-first). Every task cites requirements. Ticks mark
work actually done.

- [x] **T0** Freshness check of the source spec repository (read-only) at
  the start of work. [Principle IV]
- [x] **T1** B1: `spec.md`, `provenance.md` (pin `c65e238`; 10 claim ids and
  4 question ids verified against the import). [FR-047-01..14]
- [x] **T2** B2: `plan.md`, this file. [all]
- [x] **T3** Write the `observed_*` tests (11 cases). [FR-047-01..07;
  SC-047-1]
- [x] **T4** Write the `documented_choice_*` / `unresolved_*` tests. Register
  the test-only module, run the filter, and record the failing run
  (`UndefinedFunction`) in validation.md. [FR-047-08..14]
- [x] **T5** Implement `balance.rs`: descriptor, input checks, permutation
  and scaling phases, outputs. [FR-047-01..14]
- [x] **T6** Register: `pub mod balance;` in `factor/mod.rs`. [SC-047-2]
- [x] **T7** Run the verification commands (feature tests, `math::sparse`
  and `factor::svd` regressions, clippy, rustfmt) and the end-of-work
  freshness check; record them in validation.md. [SC-047-1..3]
- [x] **T8** Traceability table (claim → FR → test → result) in
  validation.md. [SC-047-3; Principle VI]
