# Implementation Plan — 014-figure-export (`saveas`)

## Constitution Check

- I Specification-first: both normative FRs cite approved claims
  (`saveas.signature-primary`, `saveas.creates-file`); everything else is
  explicitly documented-choice — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB
  invocation anywhere in this feature (tests are pure Rust) — PASS
- III Independence: `saveas` is an adapter over RunMat's own existing
  export pipeline; no MATLAB-internal algorithm is inferred or imitated —
  PASS
- IV Provenance: `provenance.md` complete with the authoritative batch-2
  commit pin `b054f3ad…434ef`; unresolved questions carried — PASS
- V Test-first: tasks order tests (T2–T3) before implementation (T4–T5);
  no fabricated results (validation records real runs) — PASS
- VI Traceability: FR ↔ claim ↔ test mapping recorded in `validation.md`
  at B5 — PASS (by construction)
- VII Minimal scope: one builtin, one new file + registration line + a
  minimal visibility change in `print.rs` (see below); no refactoring —
  PASS
- VIII Human gates: this plan stops at Gate 2; no implementation before
  Gate 3 approval — PASS
- IX RunMat compatibility: follows `plotting/ops/` conventions
  (descriptor consts, `runtime_builtin` macro, GPU/fusion specs, error
  builders), doc metadata per `docs/builtins/authoring.md` — PASS
- X Licensing: no external sources — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `saveas` builtin (new) | `crates/runmat-runtime/src/builtins/plotting/ops/saveas.rs` |
| Module registration | `crates/runmat-runtime/src/builtins/plotting/mod.rs` — add `#[path = "ops/saveas.rs"] pub(crate) mod saveas;` in the alphabetical ops block (between `quiver` and `scatter`) |
| Reuse enablement (minimal edit) | `crates/runmat-runtime/src/builtins/plotting/ops/print.rs` — raise visibility of `render_png` and `write_bytes` to `pub(crate)` (no behavioural change) |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (category `plotting`); feeds `docs/builtins` tooling per `authoring.md` |

Exemplar: `plotting/ops/print.rs` (the existing figure-export builtin —
descriptor consts, GPU/fusion specs, error descriptors `RM.PRINT.*` /
`RunMat:print:*`, async builtin body, atomic write, inline test module).

## Adapter design — reuse of the existing export machinery (no new rendering)

The export pipeline `print` already uses, and `saveas` will reuse
verbatim:

1. **Handle resolution**: figure handles are `FigureHandle(u32)`
   (`plotting/core/state.rs:29`), surfaced to user code as numeric double
   scalars (`figure_builtin -> BuiltinResult<f64>`, `ops/figure.rs`).
   Parse the `fig` argument with the shared machinery
   (`ops/common/handles.rs::handle_from_scalar`, accepting
   `Value::Num` / `Value::Int` / 1-element `Value::Tensor`, as
   `print.rs::figure_handle_arg` does). Unlike `print`, the export's
   confirmed form makes `fig` required — `saveas` takes an explicit
   handle first argument (documented choice: no implicit
   `current_figure_handle()` fallback for the confirmed 2-arg form,
   since only `saveas(fig, filename)` is approved).
2. **Rendering**: `print.rs::render_png(handle, width, height)` — which
   under `feature = "plot-core"` delegates to
   `crate::builtins::plotting::render_figure_snapshot` (re-export of
   `engine::render_figure_snapshot`, `plotting/mod.rs:622-629`), and
   under `not(plot-core)` returns the render error
   "plot-core support is not enabled in this build". Reuse via the
   `pub(crate)` visibility change; default width/height/DPI constants
   mirrored from `print` (800×600 @ 150 DPI equivalent).
3. **File write**: `print.rs::write_bytes(path, payload)` — atomic
   temp-file-then-rename through `runmat-filesystem` (WASM-safe), with
   temp cleanup on failure. Reused unchanged; this is what makes the
   normative "file exists and is non-empty" observable.

Note / correction to the F14 selection row: `export_figure_scene`
(`plotting/mod.rs:229`, re-exported as `runtime_plot_export_figure_scene`
in `lib.rs:375`) serializes a *replay scene payload*, not an image file.
The correct substrate for `saveas`'s observed behaviour (an image file on
disk) is the `render_figure_snapshot` + `write_bytes` path used by
`print`. `export_figure_scene` is NOT used by this feature.

`saveas.rs` therefore contains only: argument parsing (2-arg confirmed
form; 3-arg documented-choice form), extension → format resolution
(documented choice, PNG only), error descriptors (`RM.SAVEAS.*` /
`RunMat:saveas:*`: `InvalidInput`, `UnsupportedFormat`, `RenderFailed`,
`IoFailure`, `Internal`), and calls into the two reused functions.

Builtin surface, following the `print`/`drawnow` house convention for
side-effect builtins: `sink = true`, `suppress_auto_output = true`,
`accel = "metadata"`, output mode `Fixed`, internal `bool` status,
`type_resolver` returning `Type::Bool`. This models the observed
"returns no output; the effect is the written file" (FR-014-03); the
exact MATLAB semantics of output capture are unobserved and not asserted.

## `plot-core` feature gating

`plot-core` is a **default** feature of `runmat-runtime`
(`Cargo.toml:85`). Gating matches `print` exactly and comes for free by
reusing `render_png`:

- With `plot-core`: full render + write path.
- Without `plot-core`: `saveas` still registers and parses its arguments;
  the rendering step errors with the render-failure descriptor and detail
  "plot-core support is not enabled in this build". No file is created.
  (Documented choice, outside the export's scope.)

## TEST-ISOLATION and headless execution

**Headless is feasible — evidence, not assumption**: the existing test
`print_adds_png_extension_and_writes_png` (`print.rs` test module) builds
a figure with `plot_builtin`, exports a real PNG into
`std::env::temp_dir()`, asserts PNG magic bytes and `len > 1000`, and
deletes the file — in a plain `cargo test` unit test. Two mechanisms make
this work headlessly:

- the plotting test environment (`plotting::tests::ensure_plot_test_env`
  + `lock_plot_registry`) sets `RUNMAT_DISABLE_INTERACTIVE_PLOTS=1` and
  `RUNMAT_HOST_MANAGED_PLOTS=1`, so figure creation never opens a window
  (`core/state.rs:385-418`);
- `runmat-plot`'s image exporter falls back to a CPU raster path when
  headless GPU initialization is unavailable
  (`crates/runmat-plot/src/export/image.rs` — 
  `is_headless_gpu_unavailable_error` → `cpu_surface::render_figure_png_bytes`).

So the observed file-created behaviour CAN be tested headless in unit
tests, and `saveas` tests copy the `print` test pattern exactly.

Isolation rules (from `saveas.q-side-effect`, binding for every test):

- Output paths: unique per test — `std::env::temp_dir()` joined with
  `runmat_saveas_test_<stem>_<pid>_<sanitized thread name>` (the
  `unique_temp_path` pattern from `print.rs` tests), so parallel test
  threads and repeated runs never collide.
- Self-cleaning: each test removes any pre-existing file before the call
  and removes the output before finishing; assertions that can panic are
  arranged so cleanup still occurs (capture result, clean, then assert)
  in both success and failure paths. No artefacts outside temp.
- Serialization: tests take `lock_plot_registry()` + `ensure_plot_test_env()`
  + `reset_hold_state_for_run()` / `clear_figure(None)` in a `setup()`
  helper, as `print.rs` tests do, to serialize access to the global
  figure registry.
- The no-`plot-core` error path gets a narrow `#[cfg(not(feature = "plot-core"))]`
  test asserting the documented error and that no file is created; it
  only runs under `cargo test -p runmat-runtime --no-default-features`
  (recorded in validation as a separate, explicitly-invoked run — the
  default CI invocation does not exercise it).

Residual headless risk (surface at gate): the CPU fallback engages only
for errors classified as "headless GPU unavailable"; an exotic CI GPU
failure of a different class would fail the render. This risk is
identical to the one the existing `print` test already carries in CI, so
this feature adds no new risk class.

## Test plan (inline `#[cfg(test)]` in `saveas.rs`, house convention)

Normative (cite FR/claims in names or comments):

- `saveas_creates_file_and_is_nonempty` — build plot, `saveas(fig, <tmp>.png)`,
  assert file exists (case `file-created`) and `size > 0` (case
  `file-nonempty`); do NOT assert 12312 (environment-specific). Clean up.
  [FR-014-01, FR-014-02]
- `saveas_accepts_numeric_figure_handle` — explicit handle from
  `figure_builtin`/`gcf` path. [FR-014-01]
- Descriptor test — signatures cover `saveas(fig, filename)`; suppressed
  output/sink flags present. [FR-014-01, FR-014-03]

Documented-choice, tagged `unresolved_choice`:

- missing extension → `.png` appended (parse-level test, no render);
- unsupported extension (e.g. `.svg`) → `RunMat:saveas:UnsupportedFormat`;
- 3-arg form with `'png'` accepted / other formattype rejected;
- invalid handle scalar (0, negative, NaN) → error; nonexistent live
  figure → error; non-text filename → `RunMat:saveas:InvalidInput`;
- PNG magic-bytes check on a `.png` target (implementation-consistency
  only; format inference is unresolved `saveas.q-formats`).
- `#[cfg(not(feature = "plot-core"))]`: render step errors, no file.

## Risks & rollback

- Risk: headless CI render failure outside the CPU-fallback class — same
  exposure as existing `print` tests; mitigation: none needed beyond
  matching `print`; if it ever fires it fires for `print` too.
- Risk: visibility change in `print.rs` (`pub(crate)` on `render_png`,
  `write_bytes`) could drift into refactoring — scope-limited to the two
  one-word visibility edits; any signature change is out of scope and
  returns to Gate 2.
- Risk: `saveas` requiring an explicit handle diverges from a potential
  future 1-arg/other form — only the 2-arg form is approved; new forms
  return to the spec side first.
- Rollback: delete `saveas.rs`, remove the one `mod` line, revert the two
  visibility keywords. No other file is touched.

## Verification commands

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`cargo test -p runmat-runtime saveas` (default features; normative +
documented-choice tests) · `cargo test -p runmat-runtime --no-default-features saveas`
(narrow no-plot-core error-path test, explicitly recorded run)
