# Tasks — 014-figure-export (`saveas`)

Order is normative (test-first). Every task cites requirements. No task
runs before Gate 3 approval.

Status 2026-07-22: T1–T7 executed after batch-gate approval. Deviation on
T3: the planned `#[cfg(not(feature = "plot-core"))]` test is impossible
(`builtins::plotting` compiles only under `plot-core`; without it,
`saveas` — like `print` — does not exist at all) and was removed; see
`validation.md` "Deviations" for this and the other recorded deviations.

- [x] **T1** Substrate confirmation (no code changes): re-verify that
  `print.rs::render_png` / `write_bytes` signatures and
  `handles::handle_from_scalar` match the plan's adapter design, and that
  the `print_adds_png_extension_and_writes_png` test passes headlessly in
  this environment (`cargo test -p runmat-runtime print_adds_png`).
  [FR-014-01, FR-014-02; plan "Adapter design", "TEST-ISOLATION"]
- [x] **T2** Author failing normative tests in the new `saveas.rs` test
  module (red step; compilation gated by T4): 
  `saveas_creates_file_and_is_nonempty` (file exists ∧ size > 0; no exact
  byte-count assertion; unique temp path; self-cleaning in success and
  failure paths), `saveas_accepts_numeric_figure_handle`, descriptor/
  suppressed-output test. [FR-014-01, FR-014-02, FR-014-03; SC-014-1,
  SC-014-4]
- [x] **T3** Author failing documented-choice tests, each tagged
  `unresolved_choice`: missing-extension append `.png`; unsupported
  extension → `UnsupportedFormat`; 3-arg `'png'`/non-`'png'`; invalid
  handle scalar, nonexistent figure, non-text filename → errors; PNG
  magic bytes on `.png` target; `#[cfg(not(feature = "plot-core"))]`
  render-error test (no file created). [FR-014-04; SC-014-3, SC-014-4]
- [x] **T4** Implement `crates/runmat-runtime/src/builtins/plotting/ops/saveas.rs`:
  descriptor consts, GPU/fusion specs, `RM.SAVEAS.*` error descriptors,
  argument parsing (2-arg confirmed; 3-arg documented choice), extension
  → PNG resolution, calls to reused `render_png` + `write_bytes`; apply
  the two `pub(crate)` visibility edits in `print.rs`. [FR-014-01..04]
- [x] **T5** Register the module in `plotting/mod.rs` (alphabetical ops
  block) and confirm discoverability via `builtin_function_by_name`.
  [SC-014-2]
- [x] **T6** Run verification commands (fmt, check, default-feature test
  run, explicit `--no-default-features` run for the gated test); record
  real command output in `validation.md`. [SC-014-1, SC-014-2, SC-014-4;
  Principle V]
- [x] **T7** Traceability table in `validation.md`
  (claim → FR → test → result), including the unresolved-question
  register (`saveas.q-side-effect`, `saveas.q-formats`) and the list of
  documented choices awaiting spec-side resolution. [SC-014-3;
  Principle VI]
