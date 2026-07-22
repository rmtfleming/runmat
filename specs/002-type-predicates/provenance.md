# Provenance Record — 002-type-predicates

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifiers | `iscell` 0.2.0, `isstruct` 0.2.0, `isfile` 0.2.0 — all `status: approved` |
| Specification revision | Interim content pin: `specs/imports/matlab-interface-spec/SHA256SUMS` (digest `311a11d7…daae`); commit pin pending source-repo commit (waiver recorded 2026-07-22) |
| Review status | Approved 2026-07-22 by repository maintainer (`docs/review/2026-07-22-tier-a-submission.md` in source repo) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| iscell.signature-primary | public-interface-fact | yes |
| iscell.output-class | black-box-observation | yes |
| iscell.logical-cell-test | black-box-observation | yes |
| isstruct.signature-primary | public-interface-fact | yes |
| isstruct.output-class | black-box-observation | yes |
| isstruct.logical-struct-test | black-box-observation | yes |
| isfile.signature-primary | public-interface-fact | yes |
| isfile.output-class | black-box-observation | yes |
| isfile.missing-false | black-box-observation | yes |
| (summary text: isfile existing→true) | approved summary, no observation | summary-derived |

Unresolved questions carried: `iscell.q-output`, `isstruct.q-output`,
`isfile.q-existing`, `isfile.q-folder`.
