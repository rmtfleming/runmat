# Provenance Record — 014-figure-export

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `saveas` 0.2.0 — `status: approved` (observed behaviour confirmed) |
| Specification revision | Commit pin `b054f3ad809ceee28529a4dd7a6e28d2b6b434ef` of `matlab-interface-spec`, imported via packaged export `dist/runmat-export-2026-07-22` (see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, Batch 2 — authoritative pin; supersedes the batch-1 interim content pin) |
| Review status | Packaged-export review approved 2026-07-22 by repository maintainer (batch-2 record in `IMPORT-MANIFEST.md`; per-spec `status: approved` verified in `interface.yaml` at import) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |
| Import artifacts | `specs/imports/matlab-interface-spec/specifications/saveas/{interface.yaml, behaviour.md, provenance.yaml, observations/saveas_observations.json}`; drift check: `sha256sum -c SHA256SUMS` |
| Drift check result (2026-07-22, this feature) | All 4 `saveas` artifacts OK; 108/109 entries OK overall. Sole failure: `IMPORT-MANIFEST.md` itself — self-referential (the manifest gained its batch-2 section after `SHA256SUMS` was generated). No specification-content drift; housekeeping item for the import maintainer (exclude the manifest from the sums or regenerate). |

## Relied-upon claims

| claim_id | category | normative | statement (abbreviated) |
|---|---|---|---|
| saveas.signature-primary | public-interface-fact | yes | `saveas(fig, filename)` is accepted (observed; backing case `file-created`) |
| saveas.creates-file | black-box-observation | yes | Saving a graphics object creates the target file (isfile true) with nonzero byte size; saveas returns no output — the effect is the written file (backing cases `file-created`, `file-nonempty`) |

Observed values backing `saveas.creates-file`: `file-created` → logical
1×1 `true`; `file-nonempty` → double 1×1 `12312` (an off-screen line plot
saved as a PNG of about 12 kB on R2026a/GLNXA64). The byte count is
environment-specific; the normative content of the claim is
file-created and file-non-empty, and feature tests assert only existence
and `size > 0`.

## Unresolved questions carried (non-normative)

| id | question |
|---|---|
| saveas.q-side-effect | saveas requires a graphics object and writes a file; experiments must create a figure, target a temporary path, verify, and clean up (drives this feature's test-isolation rules) |
| saveas.q-formats | Enumerate supported format types and extension inference |

Additionally pending in the export (not confirmed, treated as
documented-choice territory): the three-argument form
`saveas(fig, filename, formattype)`; input rows `fig`, `filename`,
`formattype`; the "(none)" output row; the export lists no error or
warning cases.

## External implementation sources

None beyond RunMat's own codebase (existing `print` export substrate).
No external algorithm or licensed code is imported by this feature.
