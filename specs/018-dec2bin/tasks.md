# Tasks — 018-dec2bin

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Read the approved export (`interface.yaml`, `behaviour.md`,
  `provenance.yaml`, observations) and the `strings/core` exemplars
  (`int2str.rs`, `num2str.rs`); confirm value extraction and optional-arg
  patterns. [FR-018-01..06] (no code changes)
- [x] **T2** Author the four observed cases as normative tests
  (`observed_*`) plus the documented-choice tests (`unresolved_choice_*`)
  in the new file's test module (red step; compile gated by T3).
  [FR-018-01..06; SC-018-1]
- [x] **T3** Implement `dec2bin.rs`: descriptor (two signatures), GPU/fusion
  specs, error descriptors, width parsing, element extraction, binary
  formatting and padding. [FR-018-01..06]
- [x] **T4** Register the module in `strings/core/mod.rs`; verify registry
  discoverability. [SC-018-2]
- [x] **T5** Run verification commands; record output in `validation.md`.
  [SC-018-1..3]
- [x] **T6** Traceability table in `validation.md` (claim → FR → test →
  result). [SC-018-3; Principle VI]
