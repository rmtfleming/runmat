# Provenance Record — 009-sparse-construction

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifiers | `full` 0.2.0, `speye` 0.2.0, `spdiags` 0.2.0 — all `status: approved` |
| Specification revision | Interim content pin: `specs/imports/matlab-interface-spec/SHA256SUMS` (digest `311a11d7…daae`); commit pin pending source-repo commit (waiver recorded 2026-07-22) |
| Review status | Approved 2026-07-22 by repository maintainer (`docs/review/2026-07-22-tier-a-submission.md` in source repo) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| full.signature-primary | public-interface-fact | yes |
| full.output-class | black-box-observation | yes |
| full.full-preserves-values | black-box-observation | yes |
| speye.signature-primary | public-interface-fact | yes |
| speye.output-class | black-box-observation | yes |
| speye.sparse-identity | black-box-observation | yes |
| spdiags.signature-primary | public-interface-fact | yes |
| spdiags.output-class | black-box-observation | yes |
| spdiags.extract-vs-construct | black-box-observation | yes |
| (summary text: spdiags second output d = diagonal indices) | approved summary, no observation | summary-derived |
| (summary text: spdiags extraction reads sparse inputs too) | approved summary, no observation | summary-derived |

Unresolved questions carried: `full.q-class`, `full.q-complex`,
`speye.q-class`, `spdiags.q-forms`, `spdiags.q-outputs`.
