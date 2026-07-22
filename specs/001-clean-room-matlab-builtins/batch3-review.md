# Batch 3 Combined Review — Features 011–015 — 2026-07-22

Scope: 011 system, 012 urlread, 013 display, 014 saveas, 015 inputParser.
Packages authored docs-first (B1–B3) by five agents; no source changed
before this gate.

## Analysis highlights

- Provenance: all five on the authoritative commit pin `b054f3ad…`
  (dist/runmat-export-2026-07-22); zero blockers. SHA256SUMS regenerated
  excluding the self-referential IMPORT-MANIFEST.md (108/108 verify OK).
- 015 verdict: implementable now on existing machinery — exemplar
  `containers.Map` (class registration + dotted-method builtins +
  HandleObject state via `gc_with_value_mut`); no prerequisite feature.
- 013 findings for the spec side: RunMat `disp` column separator and scalar
  indent diverge from observed `display` text; display uses a local
  byte-exact formatter; recommend spec-side `disp` observations.
- 014: headless PNG export testable (precedent: print.rs test with CPU
  raster fallback); selection-row substrate pointer corrected to
  `render_figure_snapshot` + `write_bytes`.
- 012: http path is summary-derived (only file:// observed); historical
  status output deliberately omitted (absent from export).

## Gate decisions — RECORDED 2026-07-22 (maintainer, interactive gate)

- [x] **Implementation APPROVED for all five features** as planned.
- [x] `system` on Windows: **unsupported error** (no untested cmd /C path).
- [x] `system` statement calls: **forward captured stdout to console**
  (tagged unresolved_choice).
- [x] `display(X, name)`: **reject with clear error** (form is pending in
  the export; no invented header formatting).
- Publication (commit/push/PR) remains a separate approval.

## B4/B5 outcome (2026-07-22, verified by main session)

- Implemented: `system` (`io/repl_fs/`), `urlread` (`io/http/`), `display`
  (`io/`), `saveas` (`plotting/ops/`, + three permitted pub(crate)
  visibility words in `print.rs`), `inputParser` (four dotted-method
  builtins in `introspection/input_parser.rs`).
- Batch-3 module tests: **60 passed, 0 failed** (system 7, urlread 11,
  display 12, saveas 10, inputParser 20). Regression guards: disp 13/13,
  print 6/6 (agent-run).
- Full runtime lib suite: **6654 passed, 0 failed** (main session) — no
  cross-feature regressions. `cargo check` clean, `cargo fmt --check`
  clean, clippy zero warnings/errors.
- All four gate decisions implemented as recorded. Per-feature evidence in
  each feature's `validation.md`.
- Known pre-existing issue surfaced (not fixed, out of scope):
  `--no-default-features` build fails with 14 errors in plotting/geometry
  files unrelated to this batch.
- Publication (commit/push/PR): NOT performed — separate approval.
