# Provenance Record — 016-shape-storage-predicates

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifiers | `isrow` 0.2.0, `iscolumn` 0.2.0, `issparse` 0.2.0, `iscellstr` 0.2.0, `istable` 0.2.0 — all `status: approved` |
| Specification revision | Source commit `e46587ae1b78237e0f43cd55f678de01a28495a4` ("Assemble Tier B batch 2 skeletons"; Tier B batch 1 import — see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, Batch 3 section) |
| Review status | Approved 2026-07-22 by repository maintainer (source `docs/review/2026-07-22-tier-b-batch1-human-loop.md`, "all 12 approved") |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| isrow.signature-primary | public-interface-fact | yes |
| isrow.output-class | black-box-observation | yes |
| isrow.logical-row-test | black-box-observation | yes |
| iscolumn.signature-primary | public-interface-fact | yes |
| iscolumn.output-class | black-box-observation | yes |
| iscolumn.logical-column-test | black-box-observation | yes |
| issparse.signature-primary | public-interface-fact | yes |
| issparse.output-class | black-box-observation | yes |
| issparse.logical-sparse-test | black-box-observation | yes |
| iscellstr.signature-primary | public-interface-fact | yes |
| iscellstr.output-class | black-box-observation | yes |
| iscellstr.logical-cellstr-test | black-box-observation | yes |
| istable.signature-primary | public-interface-fact | yes |
| istable.output-class | black-box-observation | yes |
| istable.logical-table-test | black-box-observation | yes (true-case unreachable in RunMat — documented note in `spec.md`) |

## Notes

- Source `provenance.yaml` author for this batch is "clean-room agent
  (Claude Opus 4.8)" (Tier A batches were Claude Fable 5); approval is by
  the same repository maintainer (recorded in the import manifest).
- The `istable` observed case `table` (call `istable(array2table(...))`)
  cannot be constructed in RunMat because the runtime has no table value
  kind (Blocker B-010-1, `specs/010-table-conversion/spec.md`). The
  gate-ratified implementation is always-false, which is correct for every
  constructible input; the claim istable.logical-table-test is cited as the
  observed basis and the true-case unreachability is a documented note, not
  an implemented behaviour.

Unresolved questions carried: `isrow.q-empty`, `iscolumn.q-empty`,
`issparse.q-output`, `iscellstr.q-strings`, `istable.q-timetable`.
