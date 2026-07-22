# Provenance Record — 007-numeric-string-conversion

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifiers | `int2str` 0.2.0, `str2num` 0.2.0 — both `status: approved` |
| Specification revision | Interim content pin: `specs/imports/matlab-interface-spec/SHA256SUMS` (digest `311a11d7…daae`); commit pin pending source-repo commit (waiver recorded 2026-07-22) |
| Review status | Approved 2026-07-22 by repository maintainer (`docs/review/2026-07-22-tier-a-submission.md` in source repo) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| int2str.signature-primary | public-interface-fact | yes |
| int2str.output-class | black-box-observation | yes |
| int2str.rounding-and-char | black-box-observation | yes |
| str2num.signature-primary | public-interface-fact | yes |
| str2num.output-class | black-box-observation | yes |
| str2num.parse-eval | black-box-observation | yes |
| (summary text: str2num unparsable→empty) | approved summary, no observation | summary-derived |

Unresolved questions carried: `int2str.q-rounding`, `int2str.q-spacing`,
`str2num.q-eval`, `str2num.q-second-output`.

## Implementation sources

No external implementation code was used. Both builtins are derived from the
approved behavioural requirements and RunMat's own architecture
(`num2str`/`str2double` house patterns; the registered `eval` builtin for
delegated evaluation). The `str2num` numeric expression grammar is a
standard recursive-descent evaluator written for this feature.
