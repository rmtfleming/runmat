# Tasks — 060-exception-causes

The order is normative (test-first). A task is ticked only once done.

- [x] **T0** Freshness check of the source spec repository (read-only):
  HEAD = origin/main = `c3305a5`, no change to `specifications/{addCause,
  MException,throw}` since the pins, no draft. [Principle IV]
- [x] **T1** B1: `spec.md`, `provenance.md` (claim ids verified).
  [FR-060-01..11]
- [x] **T2** B2: `plan.md` and this file. [all]
- [x] **T3** Core type: `causes` on `runmat_value::MException`
  (`MException::new` unchanged), `with_cause`; codec struct literals
  updated. [FR-060-02, FR-060-09]
- [x] **T4** Tests first: replace the D-049-1 tests with the F60 tests in
  `exceptions_warnings_tests`; add the VM test file. Run with `addCause`
  stubbed (`RunMat:addCause:Internal`) and record the failing run.
  [FR-060-01..10; Principle V]
- [x] **T5** Implement `addCause`, `error_from_exception` /
  `ExceptionCauses`, causes in `exception_from_error`, `rethrow`, `ME.cause`,
  `ME.addCause(c)`, codec recursion. [FR-060-01..10]
- [x] **T6** Run the feature tests, surrounding suites, `runmat-value`
  tests, VM exception tests, clippy and rustfmt. [SC-060-1..3]
- [x] **T7** `validation.md`: traceability, core change list, closing
  freshness check; close D-049-1 in `specs/049-exceptions-warnings/`.
  [SC-060-4; Principle VI]
