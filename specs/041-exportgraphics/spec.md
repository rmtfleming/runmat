# Feature Specification: `exportgraphics` — write graphics content to a file

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`041-exportgraphics`; export `SPECIFY_FEATURE=041-exportgraphics`)

**Created**: 2026-10-29 (revised the same day for spec 0.3.0)

**Status**: Batch C2 — implemented from the approved `exportgraphics`
**0.3.0** export. FR-010 decision D1 = B (mixed); Gates 1–3 pre-approved.
Maintainer resume instruction: implement every form and option 0.3.0
confirms; default sizes are non-normative (keep RunMat defaults); formats
RunMat cannot render stay `UnsupportedOption`. The 0.2.0-based draft is
superseded.

**Input**: Approved Tier B export `exportgraphics` 0.3.0 (see
`provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Export to a raster file (Priority: P1)

`exportgraphics(obj, filename)` with an axes or figure handle writes the
file in the format named by the extension and returns no output.

**Acceptance Scenarios** (normative):

1. After `plot(1:3)`, `exportgraphics(gca, f)` with `f` ending `.png`
   writes `f` (PNG signature); no output. [exportgraphics.void-file-write,
   exportgraphics.targets, exportgraphics.format-by-extension]
2. `y = exportgraphics(gca, f)` errors. [exportgraphics.void-file-write,
   exportgraphics.error-output-arg]
3. `exportgraphics(gcf, f)` writes `f`. [exportgraphics.targets]
4. `.PNG`, `.jpg`, `.jpeg`, `.tif`, `.tiff`, `.gif` produce files starting
   with the PNG, JPEG `[255 216 255]`, little-endian TIFF `[73 73 42 0]` and
   `GIF89a` signatures. [exportgraphics.format-by-extension]
5. `'Resolution', 200` gives about twice the pixel width and height of
   `'Resolution', 100`. [exportgraphics.resolution]
6. Exporting different content to an existing path replaces the file.
   [exportgraphics.overwrite]

### User Story 2 - Options with warnings (Priority: P1)

1. `'ContentType','vector'` with a png or jpg target writes the file and
   warns. [exportgraphics.warning-content-png-vector]
2. `'BackgroundColor','none'` with a png target writes the file and warns.
   [exportgraphics.warning-background]
3. `'Append', true` with a png target errors.
   [exportgraphics.error-append-png]

### User Story 3 - Observed errors (Priority: P1)

Unknown extension (`.xyz`), no extension, a missing target folder, and a
first argument that is not a graphics handle (`42`) each error.
[exportgraphics.error-ext-unknown, exportgraphics.error-ext-none,
exportgraphics.error-bad-folder, exportgraphics.error-not-graphics]

### User Story 4 - Beyond RunMat's renderer (Priority: P2)

`.pdf`/`.PDF`/`.eps` (confirmed formats, but RunMat renders raster images
only), `.svg`/`.emf`, and the options `Padding`, `Colorspace` raise
`RunMat:exportgraphics:UnsupportedOption` and write nothing.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-041-01** `exportgraphics(obj, filename[, Name, Value, …])` MUST
  write the target file and return no output; requesting an output MUST
  raise `RunMat:exportgraphics:TooManyOutputs`
  [exportgraphics.void-file-write, exportgraphics.error-output-arg].
- **FR-041-02** Both an axes and a figure handle MUST be accepted
  [exportgraphics.targets]. RunMat renders per figure, so an axes target
  exports its whole parent figure (documented choice, U-041-1).
- **FR-041-03** The format MUST follow the extension, compared
  case-insensitively [exportgraphics.format-by-extension]: `png` → PNG;
  `jpg`/`jpeg` → JPEG; `tif`/`tiff` → little-endian TIFF; `gif` → GIF89a.
  RunMat renders the figure to PNG with its existing snapshot renderer
  (the `print`/`saveas` path) and re-encodes it with the `image` crate.
  The filename is used exactly as given.
- **FR-041-04** `pdf` and `eps` are confirmed formats
  [exportgraphics.format-by-extension, exportgraphics.content-type-pdf,
  exportgraphics.append-pdf], but RunMat has no vector renderer: they MUST
  raise `RunMat:exportgraphics:UnsupportedOption` and write nothing
  (maintainer rule: formats beyond what RunMat can render).
  `svg` and `emf` (exportgraphics.q-more-formats) likewise.
- **FR-041-05** Any other extension MUST raise
  `RunMat:exportgraphics:InvalidFileFormatForExport`
  [exportgraphics.error-ext-unknown]; no extension MUST raise
  `RunMat:exportgraphics:OutputFileNeedsNameAndExtension`
  [exportgraphics.error-ext-none].
- **FR-041-06** A target that cannot be written (e.g. missing folder) MUST
  raise `RunMat:exportgraphics:CannotCreateOutputFile`
  [exportgraphics.error-bad-folder]. Folders are not created.
- **FR-041-07** A first argument that is not a graphics handle (a number
  that names no figure/axes/graphics object, text, a non-scalar) MUST raise
  `RunMat:exportgraphics:GraphicsHandleMustBeFirst`
  [exportgraphics.error-not-graphics]. Graphics handles other than figure
  and axes (lines, text, legends, rulers, the root `0`) are unobserved and
  raise `RunMat:exportgraphics:UnsupportedOption` (U-041-2).
- **FR-041-08** `Resolution` (positive finite real scalar) MUST scale the
  pixel size proportionally [exportgraphics.resolution]: width
  `round(800·r/150)`, height `round(600·r/150)`, RunMat's `print`
  reference (800×600 at 150). The default (no `Resolution`) is RunMat's
  800×600 (documented choice; exportgraphics.default-size is non-normative;
  exportgraphics.q-default-resolution). A size above 8192 px MUST raise
  `RunMat:exportgraphics:InvalidInput`.
- **FR-041-09** `ContentType` MUST accept `'image'` and `'vector'`
  [exportgraphics.content-type-pdf]. With a raster target, `'vector'`
  writes the file and warns
  `RunMat:exportgraphics:ImageOutputIgnoresVectorContentType`
  [exportgraphics.warning-content-png-vector]. Other values →
  `InvalidInput`.
- **FR-041-10** `BackgroundColor` `'none'` with a raster target MUST write
  the file with RunMat's default background and warn
  `RunMat:exportgraphics:ReplacingTransparentBackgroundWithDefaultColor`
  [exportgraphics.warning-background] (the same for every raster format,
  documented choice). Other background values are unobserved →
  `UnsupportedOption`.
- **FR-041-11** `Append` (logical or numeric scalar): `false` is accepted;
  `true` with a raster target MUST raise
  `RunMat:exportgraphics:UnsupportedAppendValue` and leave the file
  unchanged [exportgraphics.error-append-png]. Other values →
  `InvalidInput`.
- **FR-041-12** An existing file MUST be replaced [exportgraphics.overwrite].
- **FR-041-13** (unresolved — exportgraphics.q-more-formats) Other option
  names (`Padding`, `Colorspace`, …) MUST raise
  `RunMat:exportgraphics:UnsupportedOption`.
- **FR-041-14** (documented choice) Option names are matched
  case-insensitively. Fewer than two arguments, a non-text or empty
  filename, a filename with NUL, an odd number of trailing arguments, or a
  non-text option name MUST raise `RunMat:exportgraphics:InvalidInput`.
  gpuArray arguments are gathered. All argument checks run before rendering,
  so a failing call writes nothing.
- **FR-041-15** (deviation D-041-1) Observed identifiers are in the
  MATLAB namespace (`MATLAB:print:…`, `MATLAB:TooManyOutputs`); RunMat uses
  `RunMat:exportgraphics:<same suffix>` (batch C2 decision D2 policy). The
  conditions are normative; the identifier text is not.

## Success Criteria *(mandatory)*

- **SC-041-1**: every observed case for a format RunMat can render
  reproduces in a normative test that writes only into a temporary
  directory.
- **SC-041-2**: `exportgraphics` is registered through `#[runtime_builtin]`,
  and `cargo test -p runmat-runtime --lib -- exportgraphics_tests` passes.
- **SC-041-3**: every FR maps to at least one test (validation.md).

## Unresolved behaviour and deviations

| id | question | disposition |
|---|---|---|
| exportgraphics.q-default-resolution | default pixel size | documented choice: 800×600 |
| exportgraphics.q-more-formats | emf, svg, Padding, Colorspace | `UnsupportedOption` |
| pdf / eps (confirmed) | vector output | `UnsupportedOption` (no vector renderer in RunMat) |
| U-041-1 (local) | axes target crops to the axes? | documented choice: whole parent figure |
| U-041-2 (local) | other graphics handle kinds, root | `UnsupportedOption` |
| U-041-3 (local) | BackgroundColor values other than `'none'` | `UnsupportedOption` |
| D-041-1 (deviation) | identifier namespace | `RunMat:exportgraphics:*` |
