# Tasks — 008-date-formatting

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Read the approved export (`interface.yaml`, `behaviour.md`,
  `provenance.yaml`, observations) and the canonical templates; confirm the
  `datetime` category registration point. [FR-008-01..05] (no code changes)
- [x] **T2** Author tests for the two observed cases (`serial-default`,
  `serial-format`) plus `unresolved_choice` tests for epoch anchoring,
  time-of-day tokens, rounding carry, string format, unknown-token and
  invalid-input errors, in the new file's test module (red step; compile
  gated by T3). [FR-008-01..05; SC-008-1]
- [x] **T3** Implement `datestr.rs`: GPU/fusion specs, descriptor, error
  descriptors, `civil_from_days` + epoch anchoring, token formatter,
  `#[runtime_builtin]` macro. [FR-008-01..05]
- [x] **T4** Register the module (`pub mod datestr;` in
  `builtins/datetime/mod.rs`). [SC-008-2]
- [x] **T5** Run verification commands; record output in `validation.md`.
  [SC-008-1..3]
- [x] **T6** Traceability table in `validation.md` (claim → FR → test →
  result); carry unresolved questions back. [SC-008-3; Principle VI]
