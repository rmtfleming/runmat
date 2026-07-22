# Tasks — 007-numeric-string-conversion

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Study exemplars (`num2str.rs`, `str2double.rs`,
  `iscell.rs`) and the `eval` builtin
  (`introspection/dynamic_workspace.rs`) to fix conventions and the
  delegation mechanism. [FR-007-01..07] (no code changes)
- [x] **T2** Author tests for the four observed `int2str` cases plus
  `unresolved_choice` tier tests (alignment, NaN/Inf, empty,
  integer/logical, rejection) in the new file's test module (red step;
  compile gated by T4). [FR-007-01..03; SC-007-1]
- [x] **T3** Author tests for the four observed `str2num` cases plus
  `summary_derived` (empty on unparsable, precedence) and
  `unresolved_choice` tier tests (commas, signs, `[]`, ragged rows,
  identifier text, rejection, string scalars). [FR-007-04..07; SC-007-1]
- [x] **T4** Implement `int2str.rs` (rounding, column layout, descriptor,
  GPU/fusion specs, macro registration) + `mod` entry. [FR-007-01..03]
- [x] **T5** Implement `str2num.rs` (text extraction, numeric grammar,
  `eval` delegation with empty fallback, descriptor, GPU/fusion specs,
  macro registration) + `mod` entry. [FR-007-04..07]
- [x] **T6** Run verification commands; record output in `validation.md`.
  [SC-007-1..3]
- [x] **T7** Traceability table in `validation.md` (claim → FR → test →
  result). [SC-007-3; Principle VI]
