# Provenance Record — 041-exportgraphics

| Field | `exportgraphics` |
|---|---|
| Source repository | `matlab-interface-spec` (local) |
| Approved specification | `exportgraphics` **0.3.0**, `status: approved` (supersedes 0.2.0) |
| Specification revision (import pin) | **`8614c5b396851cdac1d1eb58f1cf5f7fa7f8909e`** (IMPORT-MANIFEST Batch 8) |
| Review status | approved 2026-10-29 (source `docs/review/2026-10-29-tier-b-w2-widening-human-loop.md`) |
| Import date | 2026-10-29 |
| Freshness check | 2026-10-29: in `../matlab-interface-spec`, after `git fetch --all`, `HEAD` = `origin/main` = `8614c5b`; `git diff --name-only 8614c5b HEAD` and `… origin/main` for `specifications/exportgraphics` are empty; `git status --short` clean |
| Applicable release | as recorded in the import (see the imported `provenance.yaml`) |

The 0.2.0 pin (`104fb5f`, approval at `b9387dc`) and its claim
`exportgraphics.signature-primary` no longer apply (absent from 0.3.0).

## Relied-upon claims

All verified present in
`specs/imports/matlab-interface-spec/specifications/exportgraphics/provenance.yaml`
(0.3.0):

| claim_id | normative | used by |
|---|---|---|
| exportgraphics.void-file-write | yes | FR-041-01 |
| exportgraphics.error-output-arg | yes | FR-041-01 |
| exportgraphics.targets | yes | FR-041-02 |
| exportgraphics.format-by-extension | yes | FR-041-03 (raster), FR-041-04 (pdf/eps: UnsupportedOption) |
| exportgraphics.resolution | yes | FR-041-08 |
| exportgraphics.content-type-pdf | yes | FR-041-04, FR-041-09 (pdf not renderable) |
| exportgraphics.append-pdf | yes | FR-041-04 (pdf not renderable) |
| exportgraphics.overwrite | yes | FR-041-12 |
| exportgraphics.error-ext-unknown | yes | FR-041-05 |
| exportgraphics.error-ext-none | yes | FR-041-05 |
| exportgraphics.error-bad-folder | yes | FR-041-06 |
| exportgraphics.error-not-graphics | yes | FR-041-07 |
| exportgraphics.error-append-png | yes | FR-041-11 |
| exportgraphics.warning-content-png-vector | yes | FR-041-09 |
| exportgraphics.warning-background | yes | FR-041-10 |
| exportgraphics.default-size | **no** | FR-041-08 (not reproduced; RunMat default) |

All are `black-box-observation` claims backed by the imported
`observations/exportgraphics_observations.json`.

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| exportgraphics.q-default-resolution | documented choice: RunMat 800×600 |
| exportgraphics.q-more-formats | `RunMat:exportgraphics:UnsupportedOption` |

Local items U-041-1…3 and deviation D-041-1 are listed in `spec.md`.

## Source categories and independence

- Only the fifteen normative claims above are normative.
- The pixel-size formula, whole-figure export for axes targets, raster
  re-encoding via the `image` crate, option-name case-insensitivity and
  the `RunMat:` identifiers are **independently derived documented
  choices** based on RunMat's existing export path (`print`, `saveas`,
  `render_figure_snapshot`).
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository.
