# Tasks — 058-graphics-n3n4 (`findall`, `ishghandle`, `allchild`, `pcolor`, `surfl` 0.2.0)

Order is normative (test-first). Every task cites requirements.

- [x] **T0** Freshness check of `../matlab-interface-spec` against pins
  `1156c97` (findall, ishghandle) and `fc9f804` (allchild, pcolor, surfl);
  import checksums verified. [provenance]
- [x] **T1** Inspect upstream plotting: handle encoding and
  `resolve_plot_handle`; root/figure/axes/plot-child registries; axes
  metadata (title/labels/legend); `findobj`, `ishandle`, `isgraphics`;
  `get`/`set` on surfaces; `SurfacePlot` (`flatten_z`, `color_grid`,
  source data); `surf` rendering and handle registration; `view`;
  hand-written `ops/mod.rs`. A throwaway probe test recorded RunMat's
  current behaviour (Types, FontSize support, `set` on handle arrays).
  [FR-058-01..46] (no code changes kept)
- [x] **T2** Author observed normative tests for every case RunMat can
  represent (all cases of the five observation files).
  [FR-058-01..08, 10..14, 20..25, 30..37, 40..44; SC-058-1]
- [x] **T3** Author deviation / unresolved-choice tests (D-058-1 ignored
  test, unobserved forms, reflectance properties). [FR-058-09, 15, 26, 38,
  45, 46]
- [x] **T4** Register the modules with stub builtins (early error return),
  compile, run the five filters and record the failing run in
  `validation.md`. [Principle V]
- [x] **T5** Implement `findall.rs` (object tree). [FR-058-01..09]
- [x] **T6** Implement `ishghandle.rs`. [FR-058-10..15]
- [x] **T7** Implement `allchild.rs`. [FR-058-20..26]
- [x] **T8** Implement `pcolor.rs`. [FR-058-30..38]
- [x] **T9** Implement `surfl.rs`. [FR-058-40..46]
- [x] **T10** Run verification (feature filters, surrounding
  `builtins::plotting` suite, clippy, rustfmt); record in `validation.md`.
  [SC-058-2, SC-058-3]
- [x] **T11** Traceability table, deviations, finishing freshness check.
  [SC-058-1; Principle VI]
