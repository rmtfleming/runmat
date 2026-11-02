# Provenance Record — 068-web

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local) |
| Approved specification | `web` **0.2.0**, `status: approved` (completed from a 0.1.0 draft stub that was never imported into RunMat) |
| Specification revision (import pin) | **`e3642060f88131d84cb6280bc986a5cdee52d28f`** (IMPORT-MANIFEST Batch 15, N6) |
| Review status | approved 2026-11-02 (source `docs/review/2026-11-02-n6-new-specs-human-loop.md`) |
| Observations | 14 cases, local `file://` page only |
| Applicable release | R2026a (GLNXA64), as recorded in the imported `provenance.yaml` |
| Import date | 2026-11-02 (IMPORT-MANIFEST Batch 15) |
| Selection | maintainer, `specs/001-clean-room-matlab-builtins/selection-proposal-n6.md` (F68; G1–G3 pre-approved; disposition for `web.q-network`, `q-display`, `q-handle`, `q-headless`, `q-stat` and `q-error-ids` recorded there) |
| Freshness check (start) | 2026-11-02: in `../matlab-interface-spec`, after `git fetch origin`, HEAD = origin/main = `20825f8`; `git log e364206..origin/main` changes only `docs/prioritization/cobra-matlab-builtins.csv`; nothing under `specifications/web` changed and there are no drafts. The 4 imported files are byte-identical to `git show e364206:specifications/web/<file>` (`cmp`). |
| Freshness check (finish) | recorded in `validation.md` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/web/{interface.yaml,behaviour.md,provenance.yaml,observations/web_observations.json}` |

## Relied-upon claims

All 9 ids in the imported `provenance.yaml` (`grep claim_id`), all
`source_category: black-box-observation`, `status: confirmed`.

| claim_id | normative | used by |
|---|---|---|
| web.signature-primary | yes | FR-068-01 |
| web.matlab-browser | yes | FR-068-02 (handle closability: deviation D-068-4) |
| web.inputs | yes | FR-068-03 |
| web.system-browser | yes | FR-068-01, FR-068-04 |
| web.one-out | yes | FR-068-05 |
| web.error-numeric | yes | FR-068-06 |
| web.missing-file | **no** | FR-068-07 |
| web.unknown-option | **no** | FR-068-08 |
| web.handle-class | **no** | FR-068-09, D-068-4 |

## Unresolved questions carried

`web.q-warning-coverage`, `web.q-network`, `web.q-display`, `web.q-stat`,
`web.q-handle`, `web.q-cobra-nargout0`, `web.q-harness-h`,
`web.q-headless`, `web.q-error-ids`. `spec.md` records each disposition.

## Source categories and independence

- FR-068-01..06 come from black-box observations (identifier raised under
  `RunMat:`, D-068-1); FR-068-07..09 record non-normative observations;
  FR-068-10..15 are independently derived documented choices.
- The launcher uses the operating systems' public URL openers
  (`xdg-open`, `open`, `cmd /C start`) and the `url` crate (MIT/Apache-2.0,
  already a RunMat dependency) for `file://` conversion.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
