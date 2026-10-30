# Implementation Plan — 041-exportgraphics (spec 0.3.0)

## Constitution Check

- I Specification-first: FR-041-01..12 cite the fifteen normative 0.3.0
  claims; FR-041-13..15 are unresolved / documented choice / deviation.
  PASS.
- II Clean-room: imported export plus RunMat code only; MATLAB not invoked.
  PASS.
- III Independence: reuses RunMat's handle resolution and PNG snapshot
  renderer; re-encoding with the existing `image` dependency. PASS.
- IV Provenance: pinned to `8614c5b`, freshness checked. PASS.
- V Test-first: tests written, compiled and run (failing) before the
  builtin exists. PASS (see validation.md).
- VII Minimal scope: one new file plus two lines in `plotting/ops/mod.rs`;
  `print.rs`/`figure_persistence.rs` reused, not edited. PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptor, `sink = true`,
  `suppress_auto_output = true`, `accel = "metadata"` (as `print`/`saveas`);
  no integer-capability metadata. PASS.

## Affected files

| Item | Path |
|---|---|
| Builtin (new) | `crates/runmat-runtime/src/builtins/plotting/ops/exportgraphics.rs` |
| Registration | `#[path = "exportgraphics.rs"] pub(crate) mod exportgraphics;` in `crates/runmat-runtime/src/builtins/plotting/ops/mod.rs` |

## Design

1. Output count > 0 → `TooManyOutputs`. Fewer than two args → `InvalidInput`.
2. Gather all args.
3. Target: `resolve_plot_handle`; `Figure`/`Axes` → figure handle; other
   handle kinds and root → `UnsupportedOption`; resolution failure or
   non-numeric/non-scalar → `GraphicsHandleMustBeFirst`.
4. Filename text (non-empty, no NUL) else `InvalidInput`.
5. Options: even count, text names, case-insensitive: `Resolution`,
   `ContentType`, `BackgroundColor`, `Append`; other names →
   `UnsupportedOption`.
6. Extension: none → `OutputFileNeedsNameAndExtension`; raster
   (`png jpg jpeg tif tiff gif`) → format; `pdf eps svg emf` →
   `UnsupportedOption`; else `InvalidFileFormatForExport`.
7. Option semantics: `Append` true → `UnsupportedAppendValue`;
   `BackgroundColor` other than `'none'` → `UnsupportedOption`; pixel size
   from `Resolution` (cap 8192).
8. Render PNG via `render_figure_snapshot(fig, w, h, None)`; re-encode for
   JPEG/TIFF/GIF with `image`.
9. Write with `runmat_filesystem::write_async`; failure →
   `CannotCreateOutputFile`.
10. Emit warnings (after a successful write) through `crate::warnings::emit`.

## Verification commands

`cargo test -p runmat-runtime --lib -- exportgraphics_tests` ·
`cargo test -p runmat-runtime --lib -- builtins::plotting::print` ·
`rustfmt --edition 2021 --check <files>` ·
`cargo clippy -p runmat-runtime --lib --tests --message-format short | grep ops/exportgraphics.rs`
