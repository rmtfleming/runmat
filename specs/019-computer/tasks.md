# Tasks — 019-computer

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Confirm no existing `computer` builtin; confirm `CharArray`
  row vector support and `std::env::consts::{OS, ARCH}` availability.
  [FR-019-01, FR-019-03] (no code changes)
- [x] **T2** Author tests: observed mapping pair via pure
  `platform_strings("linux", "x86_64")` (host-independent, normative);
  cfg-gated live-builtin GLNXA64 pair; char-row-vector class check on any
  host. [FR-019-01, FR-019-02; SC-019-1]
- [x] **T3** Author `unresolved_choice` tests: documented platform pairs,
  unmapped fallback, case-insensitive `'ARCH'`, string-scalar option,
  unrecognized option error, non-text option error, too-many-inputs error.
  [FR-019-03, FR-019-04]
- [x] **T4** Implement `computer.rs`: `platform_strings` mapping, dispatch,
  descriptor (Fixed single output — multi-output form omitted per
  `computer.q-multi-output`), GPU/fusion specs, macro registration; add
  `pub mod computer;` to `introspection/mod.rs`. [FR-019-01..05]
- [x] **T5** Run verification commands; record output in `validation.md`.
  [SC-019-1, SC-019-2]
- [x] **T6** Traceability table in `validation.md` (claim → FR → test →
  result); carry unresolved questions. [SC-019-3; Principle VI]
