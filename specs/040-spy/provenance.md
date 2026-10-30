# Provenance Record — 040-spy

| Field | `spy` |
|---|---|
| Source repository | `matlab-interface-spec` (local) |
| Approved specification | `spy` **0.3.0**, `status: approved` (supersedes 0.2.0) |
| Specification revision (import pin) | **`8614c5b396851cdac1d1eb58f1cf5f7fa7f8909e`** (IMPORT-MANIFEST Batch 8, "Widen and re-approve Tier B C2 specs (W2)") |
| Review status | approved 2026-10-29 (source `docs/review/2026-10-29-tier-b-w2-widening-human-loop.md`) |
| Import date | 2026-10-29 |
| Freshness check | 2026-10-29: in `../matlab-interface-spec`, after `git fetch --all`, `HEAD` = `origin/main` = `8614c5b`; `git diff --name-only 8614c5b HEAD` and `… origin/main` for `specifications/spy` are empty; `git status --short` clean |
| Applicable release | as recorded in the import (see the imported `provenance.yaml`) |

The 0.2.0 pin (`104fb5f`, approval at `b52a0f7`) and its claim
`spy.signature-primary` no longer apply; that claim id is absent from 0.3.0.

## Relied-upon claims

All verified present in
`specs/imports/matlab-interface-spec/specifications/spy/provenance.yaml`
(0.3.0):

| claim_id | category | normative | used by |
|---|---|---|---|
| spy.void-plot | black-box-observation | yes | FR-040-01 |
| spy.error-output-arg | black-box-observation | yes | FR-040-01 |
| spy.one-line | black-box-observation | yes | FR-040-02 |
| spy.positions | black-box-observation | yes | FR-040-03, FR-040-10 |
| spy.axes | black-box-observation | yes | FR-040-04 (YDir part: deviation D-040-1) |
| spy.nz-label | black-box-observation | yes | FR-040-05 |
| spy.default-marker | black-box-observation | yes | FR-040-06 (property read-back: deviation D-040-2) |
| spy.linespec | black-box-observation | yes | FR-040-07 |
| spy.markersize | black-box-observation | yes | FR-040-08 |
| spy.all-zero | black-box-observation | yes | FR-040-09 |
| spy.input-types | black-box-observation | yes | FR-040-10 |
| spy.default-style-values | black-box-observation | **no** | FR-040-11 (not reproduced; RunMat defaults kept) |

Backing observations: imported `observations/spy_observations.json`, cases
`plots`, `points`, `dense`, `all-zero`, `tall`, `linespec`, `markersize`,
`linespec-markersize`, `default-style`, `output-arg`, `logical`, `char`.

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| spy.q-default-markersize | documented choice: RunMat `plot` default marker size |
| spy.q-large | complex → `RunMat:spy:UnsupportedOption`; large matrices unchanged |

Local item U-040-2 and deviations D-040-1…3 are listed in `spec.md`.

## Source categories and independence

- Only the eleven normative claims above are normative.
- LineSpec character set, error identifiers, gpuArray gathering, sparse
  acceptance beyond the observed cases, and RunMat default style values are
  **independently derived documented choices** from RunMat's own plotting
  subsystem. They are tagged in tests (`unresolved_choice_*`, `rejects_*`)
  and are not asserted as MATLAB-conformant.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository.
