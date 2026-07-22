# Provenance Record — 020-strncmpi

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `strncmpi` 0.2.0 — `status: approved` |
| Specification revision | Commit pin: `e46587ae1b78237e0f43cd55f678de01a28495a4` of `matlab-interface-spec` (Tier B batch 1; see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, batch-3 entry) |
| Review status | Approved 2026-07-22 by repository maintainer (source `docs/review/2026-07-22-tier-b-batch1-human-loop.md`, "all 12 approved") |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

Imported artefacts (byte-identical to source at the pinned commit, per the
import manifest):
`specs/imports/matlab-interface-spec/specifications/strncmpi/{interface.yaml,
behaviour.md, provenance.yaml, observations/strncmpi_observations.json}`.

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| strncmpi.signature-primary | public-interface-fact | yes |
| strncmpi.output-class | black-box-observation | yes |
| strncmpi.case-insensitive-prefix | black-box-observation | yes |

Backing observations (3 cases, all `errored: false`, class `logical`,
size 1×1): `match` — `strncmpi('Hello', 'help', 3)` → true;
`prefix-equal` — `strncmpi('abc', 'abd', 2)` → true;
`no-match` — `strncmpi('abc', 'xyz', 1)` → false.

## Independently derived inferences (not observations)

- Cell/string-array element-wise broadcasting, length-versus-`n` handling
  beyond the observed cases, missing-string handling, invalid-argument
  errors, and Unicode case folding are RunMat independent choices mirroring
  the existing `strncmp`/`strcmpi` builtins. They are tagged
  `unresolved_choice_*` in tests and are not presented as observed MATLAB
  behaviour.

Unresolved questions carried: `strncmpi.q-arrays` (behaviour with
cell-array inputs, element-wise).
