# Provenance Record — 039-svds

| Field | `svds` |
|---|---|
| Source repository | `matlab-interface-spec` (local) |
| Approved specification | `svds` **0.3.0**, `status: approved` |
| Specification revision (import pin) | **`8614c5b396851cdac1d1eb58f1cf5f7fa7f8909e`** (IMPORT-MANIFEST Batch 8, Tier B widening W2) |
| Review status | approved 2026-10-29 (source `docs/review/2026-10-29-tier-b-w2-widening-human-loop.md`, "Approve all 7") |
| Import date | 2026-10-29 |
| Freshness check | 2026-10-29: `git fetch --all`; `git diff --name-only 8614c5b HEAD` and `… origin/main` over `specifications/svds` empty; `git status --short` clean; HEAD = origin/main = `8614c5b` |
| Applicable release | as recorded in the imported `provenance.yaml` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/svds/{interface.yaml,behaviour.md,provenance.yaml,observations/svds_observations.json}` (19 cases) |
| Supersedes | 0.2.0 import (pin `104fb5f`, Batch 6) used by the first draft of this feature |

## Relied-upon claims

All 14 ids are verified present in the imported 0.3.0 `provenance.yaml`. All
are `black-box-observation`, `normative: true`.

| claim_id | cases | used by |
|---|---|---|
| svds.signature-primary | k | FR-039-01 |
| svds.output-class | k, dense, single | FR-039-01 |
| svds.largest | k, dense, order, largest | FR-039-02 |
| svds.default-count | default-8, default-2 | FR-039-03 |
| svds.k-exceeds | k-too-large, warn-k-too-large | FR-039-04 |
| svds.no-warning | warn-k | FR-039-04 |
| svds.k-zero | k-zero | FR-039-04 |
| svds.rectangular | rectangular | FR-039-02 |
| svds.smallest | smallest | FR-039-05 |
| svds.sigma-numeric | sigma-numeric | FR-039-05 |
| svds.zero-matrix | zero-matrix | FR-039-02 |
| svds.usv | usv-shapes, usv-rect | FR-039-06 |
| svds.error-char | err-char | FR-039-07 |
| svds.error-k-negative | err-k-negative | FR-039-07 |

The `s = svds(A)`, `s = svds(A, k, sigma)` and `[U,S,V] = svds(___)` forms
are `status: confirmed` in `interface.yaml` and are backed by the
default-count, smallest/largest/sigma-numeric and usv claims above.

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| svds.q-warning-coverage | no warnings emitted; only `warn-*` cases tested |
| svds.q-default-rule | documented choice `min(6, min(size(A)))` |
| svds.q-sigma-numeric | documented choice: nearest, ties to the larger value |
| svds.q-options | `RunMat:svds:UnsupportedOption` (options, other sigma, complex input) |
| svds.q-vector-signs | signs not asserted |

## Source categories and independence

- The 14 claims above are normative. The error identifiers are observed
  `MATLAB:svds:*`. RunMat emits `RunMat:svds:<same suffix>` (namespace
  decision recorded in `spec.md`; flagged for maintainer review).
- Default-count rule, sigma ordering and tie-breaking, `k` validation
  beyond the negative case, non-float rejection, the two-output form, and the
  more-than-three-outputs error are **independently derived documented
  choices**. They are tagged `documented_choice_*` in tests and are not
  asserted as MATLAB-conformant.
- Algorithm: full economy SVD through RunMat's own `svd` builtin (nalgebra
  0.32, BSD-3-Clause, an existing RunMat dependency), followed by selection.
  This is the textbook definition. No iterative-solver internals were
  inferred or imitated.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
