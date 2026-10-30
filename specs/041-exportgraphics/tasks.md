# Tasks — 041-exportgraphics (spec 0.3.0)

Order is normative (test-first). Every task cites requirements. The 0.2.0
task list is superseded; its T4 failing-run record never existed and is not
claimed.

- [x] **T0** Freshness check of `../matlab-interface-spec` against pin
  `8614c5b` (HEAD = origin/main = pin; no diff, clean status). [provenance]
- [x] **T1** Inspect upstream export paths (`print`, `saveas`,
  `render_figure_snapshot`, `resolve_plot_handle`, warnings emission,
  `image` dependency features) and confirm the PNG renderer runs
  headlessly (`builtins::plotting::print` passes). [FR-041-01..15]
- [x] **T2** Author the observed normative tests for every raster-format
  case and option case (`file-created`, `output-arg`, `target-figure`,
  `fmt-upper`/`jpg`/`jpeg`/`tif`/`tiff`/`gif`, `res-100`/`res-200`,
  `content-png-vector`/`content-jpg-vector`, `background`, `append-png`,
  `overwrite`, `err-ext-unknown`, `err-ext-none`, `err-bad-folder`,
  `err-not-graphics`), writing only into temporary directories.
  [FR-041-01..12; SC-041-1]
- [x] **T3** Author unresolved/beyond-renderer tests (pdf/PDF/eps/svg/emf,
  Padding/Colorspace, other BackgroundColor values, line/root targets).
  [FR-041-04, FR-041-07, FR-041-10, FR-041-13]
- [x] **T4** Author validation tests (arity, filename, option values).
  Compile and run the filter before the builtin exists and record the
  failing run (validation.md). [FR-041-08..11, FR-041-14]
- [x] **T5** Implement `exportgraphics.rs`. [FR-041-01..15]
- [x] **T6** Register: `mod exportgraphics;` in `plotting/ops/mod.rs`.
  [SC-041-2]
- [x] **T7** Run the verification commands; record the output in
  `validation.md`. [SC-041-1..3]
- [x] **T8** Traceability table and deviations in `validation.md`.
  [SC-041-3; Principle VI]
