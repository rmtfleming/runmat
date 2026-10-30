# Tasks — 040-spy (spec 0.3.0)

Order is normative (test-first). Every task cites requirements. The 0.2.0
task list is superseded; its T4 failing-run record never existed and is not
claimed.

- [x] **T0** Freshness check of `../matlab-interface-spec` against pin
  `8614c5b` (HEAD = origin/main = pin; no diff, clean status). [provenance]
- [x] **T1** Inspect upstream plotting (`plot`, `xlim`, `ylim`, `daspect`,
  `xlabel`, `get`, `findobj`, `find`, line-spec parser, module
  composition). [FR-040-01..14] (no code changes)
- [x] **T2** Author the observed normative tests for every 0.3.0 case RunMat
  can represent (`plots`, `output-arg`, `points`, `tall`, `all-zero`,
  `dense`, `logical`, `char`, `linespec`, `markersize`,
  `linespec-markersize`). [FR-040-01..10; SC-040-1]
- [x] **T3** Author unresolved/unobserved-form tests (complex input,
  line-style LineSpecs, reversed order, axes target, extra arguments).
  [FR-040-12, FR-040-13]
- [x] **T4** Author validation tests (non-numeric, N-D, missing A, bad
  LineSpec/marker size). Compile and run the filter before the builtin
  exists and record the failing run (validation.md: 0 passed, 30 failed
  together with 041). [FR-040-14]
- [x] **T5** Implement `spy.rs`: descriptor, error set, style parser,
  input normalisation, `find::evaluate`, plot + limits + aspect + label.
  [FR-040-01..14]
- [x] **T6** Register: `mod spy;` in the hand-written
  `plotting/ops/mod.rs`. [SC-040-2]
- [x] **T7** Run the verification commands; record the output in
  `validation.md`. [SC-040-1..3]
- [x] **T8** Traceability table (claim → FR → test → result) and
  deviations in `validation.md`. [SC-040-3; Principle VI]
