# Tasks — 022-legacy-predicates

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Inspect `Value` variants for object/handle-object and char-array
  representation; confirm `inputParser` (feature 015) is a
  `Value::HandleObject`; confirm `runmat_filesystem::metadata_async(...)`
  exposes `is_dir()`. [FR-022-01..04] (no code changes)
- [x] **T2** Author observed-case tests for `isobject` (object→true; numeric,
  char, cell → false) and `isstr` (char→true; numeric, string, cell → false)
  in the new files' test modules. [FR-022-01, FR-022-02; SC-022-1]
- [x] **T3** Author observed-case tests for `isfolder` and `isdir` (existing
  `tempdir` → true; self-cleaning missing path → false), plus
  `unresolved_choice_*` tests (a file → false; non-text → false).
  [FR-022-03, FR-022-04, FR-022-05; SC-022-1]
- [x] **T4** Implement `isobject.rs` + registration (GPU/fusion specs,
  descriptor, macro, `introspection/mod.rs` entry). [FR-022-01, FR-022-05]
- [x] **T5** Implement `isstr.rs` + registration, mirroring `ischar` exactly.
  [FR-022-02, FR-022-05]
- [x] **T6** Implement `isfolder.rs` + registration via the fs provider, with a
  shared `folder_predicate` helper. [FR-022-03, FR-022-05]
- [x] **T7** Implement `isdir.rs` + registration as a legacy alias delegating
  to `isfolder::folder_predicate`. [FR-022-04, FR-022-05]
- [x] **T8** Run verification commands (`cargo check`, targeted tests, `cargo
  fmt --all -- --check`); record output in `validation.md`. [SC-022-1, SC-022-2]
- [x] **T9** Traceability table in `validation.md` (claim → FR → test →
  result). [SC-022-3; Principle VI]
