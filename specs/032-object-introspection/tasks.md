# Tasks — 032-object-introspection

Order is normative (test-first intent). Every task cites requirements.

- [x] **T1** Inspect RunMat object machinery (`ClassDef`, `get_class`,
  `PropertyDef`/`MethodDef`, `Access`) and the `inputParser` registered class
  (feature 015); confirm the member-gathering mapping. [FR-032-01, FR-032-02]
  (no code changes)
- [x] **T2** Author `properties.rs`: descriptor, GPU/fusion specs, shared
  `class_name_of_argument` resolver, public-property gathering, N×1 cellstr
  output; tiered tests (`observed_*`, `unresolved_choice_*` incl.
  count-non-assertion). [FR-032-01, FR-032-04; SC-032-1, SC-032-3]
- [x] **T3** Author `methods.rs` reusing the resolver; public-method
  gathering; tiered tests incl. count-non-assertion. [FR-032-02, FR-032-04;
  SC-032-1, SC-032-3]
- [x] **T4** Author `what.rs` in `io/repl_fs/`: folder resolution, directory
  scan + classification, 1×1 struct with documented fields; tiered tests
  (real temp folders, no MATLAB). [FR-032-03, FR-032-04; SC-032-1]
- [x] **T5** Register all three via `pub mod` entries in
  `introspection/mod.rs` and `io/repl_fs/mod.rs`. [SC-032-2]
- [x] **T6** Run verification commands; record output in `validation.md`.
  [SC-032-1..3]
- [x] **T7** Traceability table in `validation.md` (claim → FR → test →
  result); make the value non-assertion explicit. [SC-032-3; Principle VI]
