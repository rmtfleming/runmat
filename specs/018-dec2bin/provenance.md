# Provenance Record — 018-dec2bin

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifiers | `dec2bin` 0.2.0 — `status: approved` |
| Specification revision | Commit pin: `e46587ae1b78237e0f43cd55f678de01a28495a4` of `matlab-interface-spec` (Tier B batch 1; see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, batch-3 record) |
| Review status | Approved 2026-07-22 by repository maintainer (`docs/review/2026-07-22-tier-b-batch1-human-loop.md` in source repo; "all 12 approved") |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| dec2bin.signature-primary | public-interface-fact | yes |
| dec2bin.output-class | black-box-observation | yes |
| dec2bin.binary-char | black-box-observation | yes |

Unresolved questions carried: `dec2bin.q-vector`, `dec2bin.q-negative`.

## Implementation sources

No external implementation code was used. The builtin is derived from the
approved behavioural requirements and RunMat's own architecture (the
`strings/core` house patterns `int2str`/`num2str`). Binary digit text is
produced with Rust's standard binary integer formatting; padding and
row-assembly logic is written for this feature.
