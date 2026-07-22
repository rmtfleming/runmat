# Provenance Record — 017-nonzeros

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifiers | `nonzeros` 0.2.0 — `status: approved` |
| Specification revision | Source commit `e46587ae1b78237e0f43cd55f678de01a28495a4` (Tier B batch 1; see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, batch-3 section) |
| Review status | Approved 2026-07-22 by repository maintainer (source `docs/review/2026-07-22-tier-b-batch1-human-loop.md`, "all 12 approved") |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| nonzeros.signature-primary | public-interface-fact | yes |
| nonzeros.output-class | black-box-observation | yes |
| nonzeros.column-major-order | black-box-observation | yes |

Backing observations: `specs/imports/matlab-interface-spec/specifications/`
`nonzeros/observations/nonzeros_observations.json` (cases `row`, `matrix`,
`col-major`, `sparse`; 4 independently constructed cases on R2026a,
GLNXA64).

## Unresolved questions carried

None recorded in the export (`unresolved_questions: []` in
`interface.yaml`; behaviour.md lists none). Implementation-side documented
choices for unobserved input classes (empty result shape, scalars, logical,
integer-dtype, complex, NaN, negative zero, N-D, GPU, explicit stored
zeros, non-numeric rejection) are recorded in `spec.md` (Unresolved
behaviour) and tested under the `unresolved_choice_*` tier; they are
candidates for spec-side observation in a future export.

## Notes

- Source `provenance.yaml` author for this batch: "clean-room agent
  (Claude Opus 4.8)"; approval by the repository maintainer (as recorded in
  the import manifest).
- No external implementation sources were used (Constitution Principle X):
  the implementation derives from the approved claims and RunMat's existing
  `Tensor`/`SparseTensor` value model only.
