# Tasks — 051-graphics-n2 (`hsv`, `alpha`, `tiledlayout` 0.2.0)

Order is normative (test-first). Every task cites requirements.

- [x] **T0** Freshness check of `../matlab-interface-spec` against pin
  `8f72edb` (no later commit or draft touches the three spec directories;
  import checksums verified). [provenance]
- [x] **T1** Inspect upstream plotting: colormap family and
  `current_colormap_length`; surface/patch `FaceAlpha`/`EdgeAlpha`
  storage, `set`/`get`, plot-child handle registry; handle-object pattern
  (`pan.rs`); subplot grid (`configure_subplot`, `encode_axes_handle`);
  `getfield` on handle objects; hand-written `ops/mod.rs`.
  [FR-051-01..28] (no code changes)
- [x] **T2** Author the observed normative tests for every case RunMat can
  represent: hsv `six`, `four`, `one`, `zero`, `default-size`, `range`,
  `class`, `err-negative`, `err-fraction`; alpha `scalar`, `object`,
  `clear`, `opaque`, `patch`, `edge-untouched`, `line-only`, `output-arg`,
  `err-bad-keyword`; tiledlayout `grid`, `tiles`, `flow`, `flow-tiles`,
  `spacing`, `parent`, `err-one-arg`, `err-zero`, `err-spacing`.
  [FR-051-01..04, 10..15, 20..26; SC-051-1]
- [x] **T3** Author deviation / unresolved / validation tests (`flat` and
  `interp`, flow growth rule, option handling, unobserved forms, malformed
  inputs). [FR-051-05, 06, 16, 17, 27, 28]
- [x] **T4** Add stubs (builtins that return an error), register the
  modules, compile, run the three filters and record the failing run in
  `validation.md`. [Principle V]
- [x] **T5** Implement `hsv` in `colormap_arrays.rs`. [FR-051-01..07]
- [x] **T6** Implement `alpha.rs`. [FR-051-10..17]
- [x] **T7** Implement `tiledlayout.rs` and `add_tile`. [FR-051-20..28]
- [x] **T8** Run the verification commands (feature filters, surrounding
  `builtins::plotting` suite, clippy, rustfmt); record output in
  `validation.md`. [SC-051-1..3]
- [x] **T9** Traceability table (claim → FR → test → result), deviations,
  and the finishing freshness check in `validation.md`. [SC-051-3;
  Principle VI]
