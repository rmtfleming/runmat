# Provenance Record — 019-computer

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `computer` 0.2.0 — `status: approved` |
| Specification revision | Commit pin: `e46587ae1b78237e0f43cd55f678de01a28495a4` ("Assemble Tier B batch 2 skeletons"; Tier B batch 1 import — see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, batch-3 section) |
| Review status | Approved 2026-07-22 by repository maintainer (source `docs/review/2026-07-22-tier-b-batch1-human-loop.md`, "all 12 approved") |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| computer.signature-primary | public-interface-fact | yes |
| computer.output-class | black-box-observation | yes |
| computer.platform-strings | black-box-observation | yes |

Both observation cases (`default` → `'GLNXA64'` 1×7 char; `arch` →
`'glnxa64'` 1×7 char) were captured on R2026a GLNXA64 — the observed pair is
normative only for that platform. Identifier pairs for other platforms are
independently derived documented choices (no observation backing) and are
tagged as such in the feature spec and tests.

## Unresolved questions carried

- `computer.q-multi-output` — the `[str, maxsize, endian]` multi-output form
  is not exercised; the feature omits it (Fixed single output) until the
  specification side observes it.
