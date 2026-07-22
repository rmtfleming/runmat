# Tasks — 026-append

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Study the `strcat` exemplar (text extraction, output-class
  resolution, broadcast helpers, descriptor/GPU/fusion specs) and confirm the
  two differences for `append`: two operand classes (no cell) and no
  trailing-space trimming. [FR-026-01, FR-026-02] (no code changes)
- [x] **T2** Author observed (normative) tests for the three export cases:
  `append("a","b")`, `append("x","y","z")`, `append('a','b')`. [FR-026-03;
  SC-026-1]
- [x] **T3** Author documented-choice tests (tagged `unresolved_choice_`):
  trailing whitespace preserved, mixed char+string → string, single-arg
  identity, string-array element-wise, empty-array propagation, and the three
  error conditions (zero args, non-text, size mismatch). [FR-026-04, FR-026-05]
- [x] **T4** Implement `append.rs`: variadic handler, `TextOperand`,
  `OutputKind`, broadcast-based concatenation, string/char output builders,
  descriptor + GPU/fusion specs + `#[runtime_builtin]`. [FR-026-01..05]
- [x] **T5** Register the module: `pub mod append;` in
  `strings/transform/mod.rs`. [SC-026-2]
- [x] **T6** Run verification commands (`cargo check`, scoped `cargo test`,
  `cargo fmt --all -- --check`); record output in `validation.md`. [SC-026-1..2]
- [x] **T7** Traceability table in `validation.md` (claim → FR → test →
  result). [SC-026-3; Principle VI]
