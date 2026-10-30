# Provenance Record — 037-findstr

| Field | `findstr` |
|---|---|
| Source repository | `matlab-interface-spec` (local) |
| Approved specification | `findstr` **0.3.0**, `status: approved` |
| Specification revision (import pin) | **`8614c5b396851cdac1d1eb58f1cf5f7fa7f8909e`** (IMPORT-MANIFEST Batch 8, Tier B widening W2) |
| Review status | approved 2026-10-29 (source `docs/review/2026-10-29-tier-b-w2-widening-human-loop.md`, "Approve all 7") |
| Import date | 2026-10-29 |
| Freshness check | 2026-10-29: `git fetch --all`; `git diff --name-only 8614c5b HEAD` and `… origin/main` over `specifications/findstr` empty; `git status --short` clean; HEAD = origin/main = `8614c5b` |
| Applicable release | as recorded in the imported `provenance.yaml` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/findstr/{interface.yaml,behaviour.md,provenance.yaml,observations/findstr_observations.json}` (17 cases) |
| Supersedes | 0.2.0 import (pin `104fb5f`, Batch 6) used by the first draft of this feature |

## Relied-upon claims

All 14 ids are verified present in the imported 0.3.0 `provenance.yaml`. All
are `black-box-observation`, `normative: true`.

| claim_id | cases | used by |
|---|---|---|
| findstr.signature-primary | occurrences | FR-037-01 |
| findstr.output-class | occurrences, no-match, numeric | FR-037-01 |
| findstr.start-indices | occurrences, single-char | FR-037-02 |
| findstr.order-independent | swapped | FR-037-03 |
| findstr.overlapping | overlapping | FR-037-02 |
| findstr.equal-length | equal-length-match, equal-length-nomatch | FR-037-03 |
| findstr.no-match | no-match, equal-length-nomatch, empty-pattern, empty-first, both-empty, case | FR-037-04 |
| findstr.no-warning | warn-call | FR-037-06 |
| findstr.case-sensitive | case | FR-037-04 |
| findstr.string-scalar | string-scalar | FR-037-05 |
| findstr.numeric | numeric | FR-037-05 |
| findstr.error-column-char | column-char | FR-037-07 |
| findstr.error-cell | err-cell | FR-037-07 |
| findstr.error-one-arg | err-one-arg | FR-037-07 |

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| findstr.q-warning-coverage | no warnings emitted; only `warn-call` tested |
| findstr.q-deprecated | no deprecation warning emitted |

The local unresolved item U-037-1 (unobserved argument classes →
`UnsupportedOption`) is listed in `spec.md`.

## Source categories and independence

- The 14 claims above are normative. The error identifiers are observed in
  the `MATLAB:` namespace. RunMat emits `RunMat:findstr:*` (namespace
  decision in `spec.md`; flagged for maintainer review).
- Mixed char/string, numeric scalars and empty arrays, multi-row numeric,
  other non-text classes, the more-than-two-arguments error and NaN handling
  are **independently derived documented choices**, tagged
  `documented_choice_*` / `unresolved_*` in tests.
- Algorithm: a plain left-to-right window scan (textbook naive string
  search), written for this feature.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
