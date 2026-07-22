# Tasks — 021-psi

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Read approved export (`interface.yaml`, `behaviour.md`,
  `provenance.yaml`, `observations/psi_observations.json`); verify commit
  pin and approval; record provenance + Principle X algorithm source.
  [FR-021-01..05; Principles II, IV, X]
- [x] **T2** Author tests first inside `psi.rs`'s test module: `observed_*`
  cases with 1e-12 tolerance citing psi.digamma-values / psi.output-class
  (release R2026a), `unresolved_choice_*` cases for poles, negatives,
  NaN/Inf, promotions, complex/string rejection, polygamma rejection, plus
  algorithm-identity and structural/GPU tests. [FR-021-01..05; SC-021-1,
  SC-021-3]
- [x] **T3** Implement `digamma` (A&S 6.3.5 recurrence + 6.3.18 asymptotic
  series, shift threshold 10, terms through B14) and the value-dispatch
  builtin body mirroring `gamma.rs` (descriptor, GPU spec, fusion spec,
  error descriptors, macro registration). [FR-021-01..05]
- [x] **T4** Register `pub(crate) mod psi;` in
  `crates/runmat-runtime/src/builtins/math/elementwise/mod.rs`
  (alphabetical). [SC-021-2]
- [x] **T5** Run verification commands (`cargo check`, targeted
  `cargo test`, `cargo fmt --all -- --check`); record actual output and
  computed-vs-observed deltas in `validation.md`. [SC-021-1, SC-021-2;
  Principle V]
- [x] **T6** Traceability table in `validation.md` (claim → FR → test →
  result); carry unresolved questions back to the specification side.
  [SC-021-3; Principles IV, VI]
