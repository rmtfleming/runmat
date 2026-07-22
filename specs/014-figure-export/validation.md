# Validation Report — 014-figure-export (`saveas`)

Date: 2026-07-22. All outputs below are from actual command runs in this
repository (Principle V; no fabricated results). No MATLAB was invoked.

## Files changed

| File | Change |
|---|---|
| `crates/runmat-runtime/src/builtins/plotting/ops/saveas.rs` | NEW — builtin + inline tests |
| `crates/runmat-runtime/src/builtins/plotting/mod.rs` | +2 lines: `#[path = "ops/saveas.rs"] pub(crate) mod saveas;` (alphabetical ops block) |
| `crates/runmat-runtime/src/builtins/plotting/ops/print.rs` | Visibility only: `render_png` (both cfg variants) and `write_bytes` raised to `pub(crate)`; rustfmt re-wrapped the two `render_png` signatures. No behavioural change |
| `specs/014-figure-export/validation.md` | NEW — this report |

## Verification commands and results

- `cargo check -p runmat-runtime` — **PASS** ("Finished `dev` profile …
  in 4m 04s", no warnings/errors).
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::plotting::saveas`
  — **PASS: 10 passed; 0 failed; 0 ignored** (6645 filtered out;
  finished in ~1.9–2.1s across runs). Test names as executed:
  - `builtins::plotting::saveas::tests::rendering::saveas_creates_file_and_is_nonempty`
  - `builtins::plotting::saveas::tests::rendering::saveas_accepts_numeric_figure_handle`
  - `builtins::plotting::saveas::tests::rendering::saveas_writes_png_bytes_unresolved_choice`
  - `builtins::plotting::saveas::tests::rendering::saveas_nonexistent_figure_errors_unresolved_choice`
  - `builtins::plotting::saveas::tests::saveas_descriptor_covers_approved_signature`
  - `builtins::plotting::saveas::tests::saveas_appends_png_when_extension_missing_unresolved_choice`
  - `builtins::plotting::saveas::tests::saveas_rejects_unsupported_extension_unresolved_choice`
  - `builtins::plotting::saveas::tests::saveas_formattype_png_only_unresolved_choice`
  - `builtins::plotting::saveas::tests::saveas_invalid_handle_errors_unresolved_choice`
  - `builtins::plotting::saveas::tests::saveas_invalid_arguments_error_unresolved_choice`
- Regression on the shared file:
  `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::plotting::print`
  — **PASS: 6 passed; 0 failed** (includes
  `print_adds_png_extension_and_writes_png`, the headless render+write
  exemplar).
- `cargo fmt -p runmat-runtime` then `cargo fmt --all -- --check` —
  **CLEAN** (exit 0, no diff).

Test counts: 10 total = 3 normative + 7 documented-choice (all tagged
`unresolved_choice` in the test name). Tests ran headlessly (plot test
env sets `RUNMAT_DISABLE_INTERACTIVE_PLOTS=1`; `runmat-plot` CPU raster
fallback), with unique per-test temp paths self-cleaned in success and
failure paths (metadata/bytes captured before cleanup, assertions after).

## Traceability (claim → FR → test → result)

| Claim / choice | FR | Test(s) | Result |
|---|---|---|---|
| saveas.signature-primary (public-interface-fact, normative) | FR-014-01 | `saveas_creates_file_and_is_nonempty`, `saveas_accepts_numeric_figure_handle`, `saveas_descriptor_covers_approved_signature` | PASS |
| saveas.creates-file (black-box-observation, normative; cases `file-created`, `file-nonempty`) | FR-014-02 | `saveas_creates_file_and_is_nonempty` (asserts existence ∧ `len > 0`; the observed 12312-byte count is environment-specific and deliberately NOT asserted) | PASS |
| saveas.creates-file — "returns no output; the effect is the written file" | FR-014-03 | `saveas_descriptor_covers_approved_signature` (sink + suppressed auto-display house convention; descriptor check) | PASS |
| Documented choice: PNG-only + `.png` append on missing extension (unresolved saveas.q-formats) | FR-014-04 | `saveas_appends_png_when_extension_missing_unresolved_choice`, `saveas_rejects_unsupported_extension_unresolved_choice`, `saveas_writes_png_bytes_unresolved_choice` | PASS |
| Documented choice: 3-arg `formattype` = 'png' only (export status pending) | FR-014-04 | `saveas_formattype_png_only_unresolved_choice` | PASS |
| Documented choice: invalid-handle / invalid-argument errors (export lists no error cases) | FR-014-04 | `saveas_invalid_handle_errors_unresolved_choice`, `saveas_nonexistent_figure_errors_unresolved_choice`, `saveas_invalid_arguments_error_unresolved_choice` | PASS |
| SC-014-2 registration/discoverability | — | `saveas_descriptor_covers_approved_signature` (`builtin_function_by_name("saveas")`) | PASS |

No orphan tests: every test cites an FR in its doc comment; every FR has
≥1 covering test.

## Deviations from the approved plan/tasks

1. **No-plot-core test (plan T3) is impossible and was removed.**
   Discovery during implementation: `builtins/mod.rs:23` compiles the
   entire `plotting` module only under `#[cfg(feature = "plot-core")]`.
   Without `plot-core` there is no `saveas` builtin at all — exactly as
   there is no `print` — so the ratified choice "plot-core-off builds
   error like print does" holds in a stronger form (both are absent;
   calls fail as unknown functions at dispatch), and a
   `#[cfg(not(feature = "plot-core"))]` unit test inside the plotting
   tree can never compile or run. A source comment in `saveas.rs`
   records this. Additionally, `cargo check -p runmat-runtime
   --no-default-features` currently fails with 14 **pre-existing**
   errors in `math/signal/zplane.rs`, `control/rlocus.rs`,
   `math/signal/envelope.rs`, `geometry/mod.rs` (unconditional
   references to `plotting`/`runmat_plot`) — none in this feature's
   files; not fixed here (out of scope, Principle VII).
2. **Test filter path.** The coordinator's filter
   `builtins::plotting::ops::saveas` matches 0 tests because ops files
   are mounted via `#[path]` as `builtins::plotting::saveas`. Both the
   requested command (0 matched, "ok") and the corrected filter
   (10 passed) were run and are recorded above.
3. **`render_png` visibility = three one-word edits, two functions.**
   `render_png` has two cfg-gated definitions (plot-core on/off), so the
   approved "two pub(crate) edits" touched three lines, all within
   `print.rs` as permitted, plus rustfmt line re-wrapping of the changed
   signatures. No other file outside this feature was edited.
4. **Minor addition:** registration assertion via
   `builtin_function_by_name` added to the descriptor test (covers
   SC-014-2; same pattern as existing builtins).

## Unresolved-question register (carried, non-normative)

- `saveas.q-side-effect` — drove the test-isolation design (unique temp
  paths, self-cleaning both paths, registry lock); remains open on the
  spec side.
- `saveas.q-formats` — format set and extension inference remain RunMat
  documented choices (PNG-only, `.png` append, `UnsupportedFormat`
  errors) pending spec-side observation.
- Also still pending in the export: 3-arg form confirmation, input rows,
  "(none)" output row semantics under capture, error/warning cases.

## Constitution notes

- Clean-room: implementation derived solely from the approved export and
  RunMat's own code; MATLAB was not invoked (the MATLAB MCP server
  advertised in the session environment was not used).
- Publication: nothing committed or pushed; implementation approval is
  not publication approval (Principle VIII).
- Licensing: no external implementation sources (Principle X);
  independent legal review may still be required per constitution.
