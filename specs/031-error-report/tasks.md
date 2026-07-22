# Tasks — 031-error-report

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Confirm no existing `getReport` builtin; confirm
  `Value::MException` exposes a `message` field and `CharArray::new_row`
  produces char row vectors. Confirm the MException-operating builtins
  (`throw`/`rethrow`) live under `diagnostics/`. [FR-031-01, FR-031-02]
  (no code changes)
- [x] **T2** Author normative tests: the observed case
  `getReport(MException('x:y','oops'))` → `'oops'` (char, 1×4); char-row-vector
  class check for a general message. [FR-031-01, FR-031-02; SC-031-1]
- [x] **T3** Author `unresolved_choice` tests: message-only report ignoring a
  populated stack / distinct identifier; empty message → 1×0 char; non-
  MException input → `RunMat:getReport:InvalidInput`. [FR-031-03, FR-031-04]
- [x] **T4** Implement `getReport.rs`: `Value::MException` → message char row
  dispatch, descriptor (Fixed single output, 1 signature, 1 error —
  extended-format options omitted per `getReport.q-options`), GPU/fusion
  specs, type resolver, macro registration; add
  `#[path = "getReport.rs"] pub mod get_report;` to `diagnostics/mod.rs`.
  [FR-031-01..05]
- [x] **T5** Run verification commands (check, test, fmt, clippy); record
  output in `validation.md`. [SC-031-1, SC-031-2]
- [x] **T6** Traceability table in `validation.md` (claim → FR → test →
  result); carry unresolved question `getReport.q-options`.
  [SC-031-3; Principle VI]
