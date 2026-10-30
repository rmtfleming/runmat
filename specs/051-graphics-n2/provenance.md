# Provenance Record — 051-graphics-n2

| Field | `hsv` | `alpha` | `tiledlayout` |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `hsv` **0.2.0**, `status: approved` | `alpha` **0.2.0**, `status: approved` | `tiledlayout` **0.2.0**, `status: approved` |
| Specification revision (import pin) | **`8f72edb119511f95884fb63f550e6c88682c327b`** (IMPORT-MANIFEST Batch 10, "N2") | same | same |
| Review status | approved 2026-10-30 (source `docs/review/2026-10-30-n2-new-specs-human-loop.md`, "Approve all 14") | same | same |
| Import date | 2026-10-30 | same | same |
| Applicable release | R2026a (GLNXA64), as recorded in the imported `provenance.yaml` | same | same |

Import integrity: the twelve imported files for these three specifications
pass `sha256sum -c` against `specs/imports/matlab-interface-spec/SHA256SUMS`.

## Freshness check

2026-10-30, at start: in `../matlab-interface-spec` after
`git fetch -q origin`, `origin/main` = `1156c97` (batch N3, a later commit).
`git log 8f72edb..origin/main` and `git log 8f72edb..HEAD` for
`specifications/{hsv,alpha,tiledlayout}` are empty, and
`git status --short` for those directories is clean: no newer or draft
version exists. The check is repeated at finish (see `validation.md`).

## Relied-upon claims

All verified present in the imported `provenance.yaml` files. Every claim
is `source_category: black-box-observation`, `normative: true`,
`status: confirmed`.

| claim_id | used by |
|---|---|
| hsv.signature-primary | FR-051-01 |
| hsv.output-class | FR-051-01 |
| hsv.values | FR-051-01 |
| hsv.range | FR-051-01 |
| hsv.zero | FR-051-02 |
| hsv.default | FR-051-03 |
| hsv.error-negative | FR-051-04 |
| hsv.error-fraction | FR-051-04 |
| alpha.signature-primary | FR-051-10 |
| alpha.scalar | FR-051-10 |
| alpha.keywords | FR-051-11 (`clear`, `opaque`); FR-051-16 (`flat`, `interp`: deviation D-051-2) |
| alpha.edge-untouched | FR-051-12 (tested on a patch: deviation D-051-3) |
| alpha.line-only | FR-051-13 |
| alpha.no-output | FR-051-14 |
| alpha.error-output-arg | FR-051-14 (identifier: deviation D-051-5) |
| alpha.error-bad-keyword | FR-051-15 |
| tiledlayout.signature-primary | FR-051-20 |
| tiledlayout.output-class | FR-051-20 |
| tiledlayout.grid | FR-051-21 |
| tiledlayout.tiles | FR-051-22 (through internal `add_tile`: deviation D-051-4) |
| tiledlayout.flow | FR-051-23 (growth through `add_tile`: D-051-4) |
| tiledlayout.options | FR-051-24 |
| tiledlayout.parent | FR-051-25 (`Parent` class: deviation D-051-6) |
| tiledlayout.error-one-arg | FR-051-26 |
| tiledlayout.error-zero | FR-051-26 |
| tiledlayout.error-spacing | FR-051-26 |

Backing observations: imported `observations/hsv_observations.json`
(9 cases), `observations/alpha_observations.json` (11 cases),
`observations/tiledlayout_observations.json` (9 cases).

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| hsv.q-default-with-figure | documented choice FR-051-05 |
| hsv.q-warning-coverage | no warnings raised |
| alpha.q-warning-coverage | no warnings raised |
| alpha.q-more-forms | documented choices / `UnsupportedOption` (FR-051-17) |
| tiledlayout.q-warning-coverage | no warnings raised |
| tiledlayout.q-flow-growth | documented growth rule (FR-051-27) |

Suggested follow-up for the specification side: a `nexttile` specification
(D-051-4), and observations of `get(t, …)` on a layout.

## Source categories and independence

- Only the claims above are normative. Observed `MATLAB:x:y` identifiers
  are raised as `RunMat:x:y`, except the output-request error (D-051-5).
- The HSV-to-RGB conversion (six hue sectors at S = V = 1) is a textbook
  colour-space formula, independently derived; the observations fix only
  the hue spacing `i/n` and the resulting values.
- The flow growth rule, option-value handling, error identifiers for
  unobserved conditions, and the layout's storage model are
  **independently derived documented choices**. They are tagged in tests
  (`unresolved_choice_*`, `deviation_*`, `rejects_*`) and are not asserted
  as MATLAB-conformant.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository.
