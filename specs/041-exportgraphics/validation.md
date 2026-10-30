# Validation Report — 041-exportgraphics (spec 0.3.0) — 2026-10-29

Environment as in `specs/040-spy/validation.md`: Linux x86_64,
`HDF5_DIR=~/anaconda3`, a runner wrapper, and a target directory shared
with other agents. MATLAB was not invoked. Every test writes only into a
`tempfile::tempdir()`, which is removed on drop.

## Freshness (T0)

`../matlab-interface-spec`: `HEAD` = `origin/main` = `8614c5b…`; no diff
for `specifications/exportgraphics` against either; clean status.

## Test-first evidence (T4)

Compiled with the module registered and no builtin present, the combined
run `-- spy_tests exportgraphics_tests` gave **0 passed, 30 failed**. All
16 `exportgraphics_tests` failed because the builtin was absent: either a
panic on `Undefined function: exportgraphics` or an identifier assertion
whose left side was `RunMat:UndefinedFunction`.

## Commands and results (T7)

- `cargo test -p runmat-runtime --lib -- spy_tests exportgraphics_tests`:
  **30 passed, 0 failed** (16 `exportgraphics_tests`):
  - `observed_file_created_from_axes`
  - `observed_output_request_errors`
  - `observed_figure_target`
  - `observed_raster_format_by_extension`
  - `observed_resolution_scales_pixels`
  - `observed_vector_content_type_on_image_warns_and_writes`
  - `observed_background_none_warns_and_writes`
  - `observed_append_true_with_png_errors`
  - `observed_overwrite_replaces_file`
  - `observed_unknown_extension_errors`
  - `observed_missing_extension_errors`
  - `observed_missing_folder_errors`
  - `observed_non_graphics_first_argument_errors`
  - `unresolved_choice_vector_formats_unsupported`
  - `unresolved_choice_unobserved_options_unsupported`
  - `rejects_invalid_arguments`
- Plotting regression (`print`, `figure_persistence`, `get`, `findobj`):
  **45 passed, 0 failed**.
- `cargo clippy -p runmat-runtime --lib --tests`: exit 0 and no warnings,
  after one fix in this file (`is_multiple_of`).
- `rustfmt --edition 2021 --check` on the three touched files: clean.

## Traceability (claim → FR → test → result) (T8)

| Claim (0.3.0) | FR | Test(s) | Result |
|---|---|---|---|
| exportgraphics.void-file-write | FR-041-01 | `observed_file_created_from_axes` (isfile, no output) | PASS |
| exportgraphics.error-output-arg | FR-041-01 | `observed_output_request_errors` | PASS (`RunMat:exportgraphics:TooManyOutputs`, D-041-1) |
| exportgraphics.targets | FR-041-02 | `observed_file_created_from_axes`, `observed_figure_target` | PASS |
| exportgraphics.format-by-extension (png, PNG, jpg, jpeg, tif, tiff, gif) | FR-041-03 | `observed_raster_format_by_extension` (signature bytes, as-given name) | PASS |
| exportgraphics.format-by-extension (pdf, PDF, eps) | FR-041-04 | `unresolved_choice_vector_formats_unsupported` | **UnsupportedOption** (no vector renderer) |
| exportgraphics.content-type-pdf | FR-041-04, FR-041-09 | pdf rejected as above; `'image'` and `'vector'` both accepted for raster in `observed_vector_content_type_on_image_warns_and_writes` | partial (pdf not renderable) |
| exportgraphics.append-pdf | FR-041-04 | pdf rejected as above | **UnsupportedOption** |
| exportgraphics.resolution | FR-041-08 | `observed_resolution_scales_pixels` (ratio 2 ± 0.05, from PNG IHDR) | PASS |
| exportgraphics.warning-content-png-vector | FR-041-09 | `observed_vector_content_type_on_image_warns_and_writes` (png, jpg) | PASS |
| exportgraphics.warning-background | FR-041-10 | `observed_background_none_warns_and_writes` | PASS |
| exportgraphics.error-append-png | FR-041-11 | `observed_append_true_with_png_errors` (file unchanged; `Append false` accepted) | PASS |
| exportgraphics.overwrite | FR-041-12 | `observed_overwrite_replaces_file` | PASS |
| exportgraphics.error-ext-unknown | FR-041-05 | `observed_unknown_extension_errors` (`.xyz`) | PASS |
| exportgraphics.error-ext-none | FR-041-05 | `observed_missing_extension_errors` | PASS |
| exportgraphics.error-bad-folder | FR-041-06 | `observed_missing_folder_errors` | PASS |
| exportgraphics.error-not-graphics | FR-041-07 | `observed_non_graphics_first_argument_errors` (unused number, text, vector) | PASS |
| exportgraphics.default-size (non-normative) | FR-041-08 | not asserted; RunMat 800×600 | n/a |
| exportgraphics.q-more-formats | FR-041-04, FR-041-13 | `unresolved_choice_vector_formats_unsupported` (svg, emf), `unresolved_choice_unobserved_options_unsupported` (Padding, Colorspace) | PASS |
| U-041-2, U-041-3 | FR-041-07, FR-041-10 | `unresolved_choice_unobserved_options_unsupported` (line/root targets, other backgrounds) | PASS |
| documented choice: validation | FR-041-14 | `rejects_invalid_arguments` | PASS |

The observed `err-not-graphics` case used `42`. The test uses an unused
handle number (`987654`) instead, because RunMat numbers figures from 1
and a long test run can create figure 42.

## Deviations

- **pdf/eps (and the pdf-only claims content-type-pdf and append-pdf) are
  not honoured.** RunMat's only figure renderer is the raster snapshot, so
  these raise `RunMat:exportgraphics:UnsupportedOption` (maintainer rule).
- **D-041-1: identifier namespace.** The observed identifiers keep their
  suffixes but move to the `RunMat:exportgraphics:` namespace:
  `InvalidFileFormatForExport`, `OutputFileNeedsNameAndExtension`,
  `CannotCreateOutputFile`, `GraphicsHandleMustBeFirst`,
  `UnsupportedAppendValue`, `TooManyOutputs`, and the warnings
  `ImageOutputIgnoresVectorContentType` and
  `ReplacingTransparentBackgroundWithDefaultColor`.
- Axes targets export the whole parent figure (U-041-1).
- Pixel sizes follow RunMat's `print` reference: 800×600 at 150, scaled
  linearly by `Resolution`. They do not match the observed pixel counts,
  which the spec marks as display-dependent and non-normative.

## Finding F-1

The same finding as `specs/040-spy/validation.md`: builtins are registered
twice, so a failing dispatcher call is retried and the retry can report a
different error. Here, after the first run failed correctly, the retry
failed to resolve the `gca` handle and reported
`GraphicsHandleMustBeFirst`. The error-path tests therefore call
`exportgraphics_builtin` directly. Success paths go through
`call_builtin_async`.

## Not verified headlessly

- Only the PNG signature, the PNG IHDR size, and the JPEG/TIFF/GIF
  signature bytes are checked; image content is not compared.
- The snapshot renderer did run headlessly in the test environment, so the
  files written are real renders.
