# Provenance Record — 005-struct-cell-access

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifiers | `fields` 0.2.0, `num2cell` 0.2.0 — both `status: approved` |
| Specification revision | Interim content pin: `specs/imports/matlab-interface-spec/SHA256SUMS` (digest `311a11d7…daae`); commit pin pending source-repo commit (waiver recorded 2026-07-22) |
| Review status | Approved 2026-07-22 by repository maintainer (`docs/review/2026-07-22-tier-a-submission.md` in source repo) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| fields.signature-primary | public-interface-fact | yes |
| fields.output-class | black-box-observation | yes |
| fields.cell-fieldnames | black-box-observation | yes |
| (summary text: fields is a legacy equivalent of fieldnames) | approved summary, no observation | summary-derived |
| num2cell.signature-primary | public-interface-fact | yes |
| num2cell.output-class | black-box-observation | yes |
| num2cell.cell-grouping | black-box-observation | yes |
| (claim statement: dim groups elements along that dimension — cell contents) | approved claim text, contents unobserved | summary-derived |

Unresolved questions carried: `fields.q-alias`, `num2cell.q-shape`.
