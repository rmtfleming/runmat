# Tasks — 020-strncmpi

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Re-read the approved export (`interface.yaml`, `behaviour.md`,
  `provenance.yaml`, observations) and the exemplars
  (`strings/core/strncmp.rs`, `strings/core/strcmpi.rs`); confirm reusable
  machinery (`TextCollection`, `logical_result`, broadcast helpers,
  `logical_text_match_type`). [FR-020-01..04] (no code changes)
- [x] **T2** Author `spec.md`, `provenance.md` (commit pin
  `e46587ae1b78237e0f43cd55f678de01a28495a4`), `plan.md`, `tasks.md`.
  [Principles I, IV]
- [x] **T3** Author the test module for `strncmpi`: three `observed_*`
  tests reproducing the export cases exactly, plus `unresolved_choice_*`
  tests for the documented independent choices (arrays, lengths, missing,
  errors) and infrastructure tests (type resolver, wgpu-gated gather).
  Tests live in the new file's `#[cfg(test)]` module, so the red step is
  compile-gated with T4. [FR-020-01..04; SC-020-1, SC-020-3]
- [x] **T4** Implement `strncmpi.rs` mirroring `strncmp.rs` (descriptor,
  errors, GPU/fusion specs, macro registration, prefix walk) with
  per-character lowercase folding per `strcmpi`. [FR-020-01..04]
- [x] **T5** Register the module: add `pub mod strncmpi;` to
  `strings/core/mod.rs`. [SC-020-2]
- [x] **T6** Run verification commands (`cargo check`, targeted
  `cargo test`, `cargo fmt --all -- --check`); record actual output in
  `validation.md`. [SC-020-1, SC-020-2]
- [x] **T7** Traceability table in `validation.md` (claim → FR → test →
  result); carry `strncmpi.q-arrays` back to the specification side.
  [SC-020-3; Principle VI]
