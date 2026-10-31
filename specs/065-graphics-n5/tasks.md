# Tasks — 065-graphics-n5 (`fplot`, `rectangle`, `validatecolor` 0.2.0)

Order is normative (test-first). Every task cites requirements.

- [x] **T0** Freshness check of `../matlab-interface-spec` against pin
  `1c91aa1`; import checksums and byte identity verified. [provenance]
- [x] **T1** Inspect upstream plotting: `fcontour`/`fsurf` sampling and
  callback calls (`call_function`), `plot`/`line` line construction and
  colour order, the line-style parser and colour table in `style.rs`,
  `PatchPlot`, `render_active_plot` vs `append_active_plot`, axis-limit
  state (auto limits query as NaN), member access on numeric handles and on
  handle objects (`tiledlayout` precedent), `warnings::emit` and
  `lastwarn`, the host eval hook. [FR-065-01..36]
- [x] **T2** Author observed normative tests for every case RunMat can
  represent (all cases of the three observation files).
  [FR-065-01..09, 20..26, 30..35; SC-065-1]
- [x] **T3** Author deviation / unresolved-choice tests (D-065-3, D-065-4,
  D-065-5; sampling, limits, geometry, unobserved inputs).
  [FR-065-10, 27, 36]
- [x] **T4** Register the modules with stub builtins (early error return,
  marker `F65-STUB`), compile, run the three filters and record the
  failing run in `validation.md`. [Principle V]
- [x] **T5** Implement `validatecolor.rs` (shared single-colour parser).
  [FR-065-30..36]
- [x] **T6** Implement `fplot.rs`. [FR-065-01..10]
- [x] **T7** Implement `rectangle.rs`. [FR-065-20..27]
- [x] **T8** Run verification (feature filters, surrounding
  `builtins::plotting` suite, clippy, rustfmt); record in `validation.md`.
  [SC-065-2, SC-065-3]
- [x] **T9** Traceability table, deviations, finishing freshness check.
  [SC-065-1; Principle VI]

## Analysis (B3)

- Coverage: every normative claim in `provenance.md` maps to an FR and at
  least one `observed_*` test (see the plan's test list); the two
  non-normative default-colour claims are deliberately not asserted.
- Orphans: none — each `unresolved_choice_*`/`deviation_*` test cites
  FR-065-10, 27, 36 or a D-065-k.
- Consistency: identifiers in spec.md, the error descriptors and the tests
  agree; the `MATLAB:hg:…`, `MATLAB:unrecognizedStringChoice` identifiers
  are reproduced under `RunMat:` and flagged (q-error-ids).
- Risk: rectangle's object model (handle object + patch) splits the
  observed "one Rectangle child" into a patch child (D-065-5).
