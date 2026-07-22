# Provenance Record — 025-normalize

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `normalize` 0.2.0 — `status: approved` |
| Specification version | 0.2.0 |
| Specification revision (pin) | source commit **`4445294`** ("Approve Tier B batch 2 after human-loop review"); Tier B batch 2; `IMPORT-MANIFEST.md` Batch 4. Verified byte-identical to source HEAD `37087db` (adds only unrelated draft batch-3 skeletons). |
| Review status | Approved 2026-07-22 (source `docs/review/2026-07-22-tier-b-batch2-human-loop.md`, "all 12 approved") |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |
| Backing observations | `observations/normalize_observations.json` (2 cases: `zscore`, `range`) |

## Relied-upon claims

| claim_id | statement (abbrev.) | category | normative |
|---|---|---|---|
| normalize.signature-primary | `N = normalize(A)` is accepted | public-interface-fact | yes |
| normalize.output-class | output class is `double` for observed inputs | black-box-observation | yes |
| normalize.zscore-and-range | default = z-score `[1 2 3]→[-1 0 1]`; `'range'` `→[0 0.5 1]` | black-box-observation | yes |

Interface-side rule ids that map onto the claims above: `normalize.out-type`,
`normalize.output-class-rule`, `normalize.zscore-and-range-rule`
(interface.yaml, all `status: confirmed`).

## Unresolved questions carried

| id | question | disposition |
|---|---|---|
| normalize.q-dim | operating dimension for matrices | implemented by documented choice (first non-singleton default; explicit `dim` honoured) — FR-025-04 |
| normalize.q-methods | other methods (norm, center, scale, medianiqr) | `center`/`scale`/`norm`/custom `range` implemented by documented choice; robust/median/MAD/IQR rejected — FR-025-05, FR-025-06 |

## Clean-room notes

- No MATLAB was invoked in this repository. All behavioural requirements trace
  to the approved export artefacts listed above.
- Sample-std (N−1) is an independently derived interpretation of the observed
  `[1 2 3] → [-1 0 1]` result (variance `2/2 = 1`), consistent with the
  standard definition of the sample standard deviation; it is not copied from
  MathWorks material.
- Independent legal review MAY still be required; this record does not
  substitute for legal advice.
