# Tasks — 049-exceptions-warnings (specs 0.2.0)

Order is normative (test-first). Every task cites requirements. Only work
actually done is ticked.

- [x] **T0** Freshness check of the source repository for `MException`,
  `lastwarn` (pin `8f72edb`) and `throw` (pin `1156c97`). [provenance]
- [x] **T1** Inspect upstream: `runmat_value::MException`,
  `runtime_error::exception_from_error`, `rethrow`, `error`, `getReport`,
  `warnings::{policy,emission}`, object member/method routes for
  `Value::MException`, value codec cause handling. [FR-049-01..12]
- [x] **T2** Write `exceptions_warnings.rs` with the four builtins
  registered as stubs ("not yet implemented") plus the full test module;
  add the `lastwarn` state accessors to `policy.rs` (needed to compile);
  hand-add `pub mod exceptions_warnings;` to `diagnostics/mod.rs`.
  [FR-049-01..13; SC-049-1]
- [x] **T3** Run the test filter against the stubs; record the failing run
  in `validation.md`.
- [x] **T4** Implement `MException`. [FR-049-01..03, U-049-1]
- [x] **T5** Implement `throw` and the `ME.throw` / `ME.throw()` / `ME.cause`
  routes. [FR-049-04, FR-049-05, U-049-2, D-049-2]
- [x] **T6** `addCause`: explicit UnsupportedOption. [FR-049-06, U-049-3,
  D-049-1]
- [x] **T7** Implement `lastwarn`; record every emitted warning in
  `emission.rs`. [FR-049-07..12, U-049-4..6]
- [x] **T8** Verification commands (feature tests, diagnostics / warnings /
  object suites, rustfmt, clippy); record in `validation.md`. [SC-049-2,
  SC-049-4]
- [x] **T9** Traceability table, deviations and final freshness check in
  `validation.md`. [SC-049-3]
