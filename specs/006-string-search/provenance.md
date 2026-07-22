# Provenance Record — 006-string-search

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifiers | `matches` 0.2.0, `strmatch` 0.2.0, `strtok` 0.2.0 — all `status: approved` |
| Specification revision | Interim content pin: `specs/imports/matlab-interface-spec/SHA256SUMS` (digest `311a11d7…daae`); commit pin pending source-repo commit (waiver recorded 2026-07-22) |
| Review status | Approved 2026-07-22 by repository maintainer (`docs/review/2026-07-22-tier-a-submission.md` in source repo) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| matches.signature-primary | public-interface-fact | yes |
| matches.output-class | black-box-observation | yes |
| matches.scalar-membership | black-box-observation | yes |
| strmatch.signature-primary | public-interface-fact | yes |
| strmatch.output-class | black-box-observation | yes |
| strmatch.index-vector | black-box-observation | yes |
| strtok.signature-primary | public-interface-fact | yes |
| strtok.output-class | black-box-observation | yes |
| strtok.leading-token | black-box-observation | yes |
| (summary text: strtok second output returns the remainder) | approved summary, no observation | summary-derived |

Unresolved questions carried: `matches.q-release`,
`matches.q-string-vs-char`, `strmatch.q-deprecated`, `strmatch.q-shape`,
`strtok.q-remainder`, `strtok.q-empty`.
