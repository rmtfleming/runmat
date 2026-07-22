# Tasks — 033-environment-identity

Order is normative (test-first). Every task cites requirements. The binding
gate constraint (RunMat-honest identity; MATLAB values NON-NORMATIVE) applies
to every task.

- [x] **T1** Confirm no existing `version`/`ver`/`verLessThan`/`license`
  builtin (grep registry + introspection dir); confirm `CharArray`,
  `StructValue`, `CellArray`, `Value::Bool`, `Value::Num`, and
  `env!("CARGO_PKG_VERSION")` (`0.5.6`) availability. [FR-033-01..07] (no code
  changes)
- [x] **T2** Author `contract_*` tests for the observable contract with
  RunMat's own values, including the identity-divergence tests
  (`version` char contains `0.5.6`; `version('-release')` != `'2026a'`;
  `verLessThan('matlab','1.0')` == `true`; `ver('MATLAB')` empty). [FR-033-01,
  02, 03, 05, 06; SC-033-1]
- [x] **T3** Author `documented_choice_*` tests: option case-insensitivity and
  error identifiers; `ver` empty-struct-array + toolbox-name independence;
  pure `version_less_than` semantics; `license` no-arg/`inuse`/`checkout`.
  [FR-033-02, 04, 05, 07]
- [x] **T4** Implement `version.rs` (shared identity constants + `option_text`)
  + registration. [FR-033-01, 02]
- [x] **T5** Implement `ver.rs` (RunMat struct + empty-struct-array choice) +
  registration. [FR-033-03, 04]
- [x] **T6** Implement `verLessThan.rs` (independent `parse_version` +
  `version_less_than`) + registration (module `ver_less_than`). [FR-033-05]
- [x] **T7** Implement `license.rs` (open-source availability + documented
  forms) + registration. [FR-033-06, 07]
- [x] **T8** Run verification commands (check, tests, fmt, clippy); record
  output in `validation.md`. [SC-033-2]
- [x] **T9** Traceability table in `validation.md` distinguishing
  contract-claims (honored) from MATLAB-value-claims (deliberately not
  asserted); carry unresolved questions. [SC-033-3; Principle VI]
