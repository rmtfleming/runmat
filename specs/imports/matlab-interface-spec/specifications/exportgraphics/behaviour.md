# `exportgraphics` — behaviour specification

- **Spec version:** 0.3.0
- **Status:** approved 2026-10-29 (Tier B widening W2, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/exportgraphics_observations.json`](observations/exportgraphics_observations.json)

## Summary

Save the contents of an axes or figure to a file whose format is chosen by the file extension; returns no output.

## Confirmed invocation forms

- `exportgraphics(obj, filename)` — Export axes or figure `obj` to `filename`. *(e.g. case `file-created`)*
- `exportgraphics(obj, filename, Name, Value)` — Export with options. *(e.g. case `res-200`)*

## Name-value arguments

- `Resolution` — Output resolution for image formats; 200 gave about twice the pixel width and height of 100 (png).
- `ContentType` — 'vector' or 'image'; both accepted for pdf; 'vector' with png or jpg writes the file and warns. Accepted: `'vector'`, `'image'`.
- `BackgroundColor` — Background colour; 'none' with png writes the file and warns.
- `Append` — Logical; true with an existing pdf succeeds and the file grows; with png it errors. Default: `false`.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — exportgraphics writes the target file and returns no output; requesting an output errors. *(cases: `file-created`, `output-arg`)*
2. **Confirmed** — The file format follows the extension (upper-case .PNG and .PDF behaved as lower-case): .png/.PNG start with the PNG signature [137 80 78 71 13 10 26 10]; .jpg/.jpeg with JPEG [255 216 255]; .tif/.tiff with little-endian TIFF [73 73 42 0]; .gif with 'GIF89a'; .pdf with '%PDF-'; .eps with '%!PS-Adobe'. *(cases: `file-created`, `fmt-upper`, `fmt-jpg`, `fmt-jpeg`, `fmt-tif`, `fmt-tiff`, `fmt-gif`, `fmt-pdf`, `fmt-upper-pdf`, `fmt-eps`)*
3. **Confirmed** — Both an axes and a figure handle are accepted as the target. *(cases: `file-created`, `target-figure`)*
4. **Confirmed** — For png, Resolution 200 produced an image about twice the width and height of Resolution 100 (727x575 vs 366x292). *(cases: `res-100`, `res-200`)*
5. **Confirmed** — For pdf, ContentType 'vector' and 'image' are both accepted. *(cases: `content-vector`, `content-image`)*
6. **Confirmed** — Append true on an existing pdf succeeds and grows the file. *(cases: `append-pdf`)*
7. **Confirmed** — Exporting different content to an existing path succeeds and replaces the file (its bytes changed). *(cases: `overwrite`)*

## Edge cases (observed)

- *(non-normative)* Observed on this machine only: a 400x300-pixel figure exported at default resolution gave a png whose pixel size lay between the Resolution 100 and 200 sizes. Pixel sizes are expected to depend on the display and figure layout (not exercised). *(cases: `file-created`, `res-100`, `res-200`)*

## Warnings (confirmed)

| condition | identifier | case |
|---|---|---|
| ContentType 'vector' with an image-format target (png and jpg observed); the file is still written. | `MATLAB:print:ImageOutputIgnoresVectorContentType` | `content-png-vector`, `content-jpg-vector` |
| BackgroundColor 'none' with a png target; the file is still written. | `MATLAB:print:ReplacingTransparentBackgroundWithDefaultColor` | `background` |

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The extension is not a recognised export format. | `MATLAB:print:InvalidFileFormatForExport` | `err-ext-unknown` |
| The filename has no extension. | `MATLAB:print:OutputFileNeedsNameAndExtension` | `err-ext-none` |
| The target folder does not exist. | `MATLAB:print:CannotCreateOutputFile` | `err-bad-folder` |
| The first argument is not a graphics handle. | `MATLAB:print:GraphicsHandleMustBeFirst` | `err-not-graphics` |
| Append true with a png target. | `MATLAB:print:UnsupportedAppendValue` | `append-png` |
| An output argument is requested. | `MATLAB:TooManyOutputs` | `output-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `file-created` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[137 80 78 71 13 10 26 10]; image_format='png'; width=546; height=434 |
| `fmt-jpg` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[255 216 255 224 0 16 74 70]; image_format='jpg'; width=546; height=434 |
| `fmt-jpeg` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[255 216 255 224 0 16 74 70]; image_format='jpg'; width=546; height=434 |
| `fmt-tif` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[73 73 42 0 46 88 0 0]; image_format='tif'; width=546; height=434 |
| `fmt-tiff` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[73 73 42 0 46 88 0 0]; image_format='tif'; width=546; height=434 |
| `fmt-gif` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[71 73 70 56 57 97 34 2]; image_format='GIF'; width=546; height=434 |
| `fmt-upper-pdf` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[37 80 68 70 45 49 46 52]; image_format='' |
| `content-jpg-vector` | `struct` | `[1, 1]` | isfile=true; warning_id='MATLAB:print:ImageOutputIgnoresVectorContentType'; first_bytes=[255 216 255 224 0 16 74 70]; image_format='jpg'; width=546; height=434 |
| `fmt-pdf` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[37 80 68 70 45 49 46 52]; image_format='' |
| `fmt-eps` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[37 33 80 83 45 65 100 111]; image_format='' |
| `fmt-upper` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[137 80 78 71 13 10 26 10]; image_format='png'; width=546; height=434 |
| `err-ext-unknown` | — | — | error `MATLAB:print:InvalidFileFormatForExport` |
| `err-ext-none` | — | — | error `MATLAB:print:OutputFileNeedsNameAndExtension` |
| `res-100` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[137 80 78 71 13 10 26 10]; image_format='png'; width=366; height=292 |
| `res-200` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[137 80 78 71 13 10 26 10]; image_format='png'; width=727; height=575 |
| `target-figure` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[137 80 78 71 13 10 26 10]; image_format='png'; width=546; height=434 |
| `content-vector` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[37 80 68 70 45 49 46 52]; image_format='' |
| `content-image` | `struct` | `[1, 1]` | isfile=true; warning_id=''; first_bytes=[37 80 68 70 45 49 46 52]; image_format='' |
| `content-png-vector` | `struct` | `[1, 1]` | isfile=true; warning_id='MATLAB:print:ImageOutputIgnoresVectorContentType'; first_bytes=[137 80 78 71 13 10 26 10]; image_format='png'; width=546; height=434 |
| `background` | `struct` | `[1, 1]` | isfile=true; warning_id='MATLAB:print:ReplacingTransparentBackgroundWithDefaultColor'; first_bytes=[137 80 78 71 13 10 26 10]; image_format='png'; width=546; height=434 |
| `append-pdf` | `logical` | `[1, 1]` | true |
| `append-png` | — | — | error `MATLAB:print:UnsupportedAppendValue` |
| `overwrite` | `struct` | `[1, 1]` | isfile=true; changed=true |
| `err-bad-folder` | — | — | error `MATLAB:print:CannotCreateOutputFile` |
| `err-not-graphics` | — | — | error `MATLAB:print:GraphicsHandleMustBeFirst` |
| `output-arg` | — | — | error `MATLAB:TooManyOutputs` |

## Unresolved questions

- `exportgraphics.q-default-resolution` — The default resolution and exact pixel sizes depend on the display and layout and are not specified.
- `exportgraphics.q-more-formats` — Other formats (e.g. emf, svg) and options (Padding, Colorspace) were not exercised.

Resolved in 0.3.0: `exportgraphics.q-formats`, `exportgraphics.q-side-effect`.
