# Provenance Record — 065-graphics-n5

| Field | `fplot` | `rectangle` | `validatecolor` |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `fplot` **0.2.0**, `status: approved` | `rectangle` **0.2.0**, approved | `validatecolor` **0.2.0**, approved |
| Specification revision (import pin) | **`1c91aa1c9bda12a34792d49dff22727d9fcbcc20`** (IMPORT-MANIFEST Batch 14, "N5") | same | same |
| Review status | approved 2026-10-30 (batch N5 human-loop review) | same | same |
| Import date | 2026-10-31 | same | same |
| Applicable release | R2026a (GLNXA64), per the imported `provenance.yaml` | same | same |

Import integrity: the 12 imported files for these three specifications pass
`sha256sum -c` against `specs/imports/matlab-interface-spec/SHA256SUMS`, and
are byte-identical to `../matlab-interface-spec/specifications/<fn>/`.

## Freshness check

2026-10-31, at start: in `../matlab-interface-spec` after
`git fetch -q origin`, `HEAD` = `origin/main` = `1c91aa1` (the pin).
`git diff --stat 1c91aa1 origin/main --
specifications/{fplot,rectangle,validatecolor}` is empty; `git status
--short` shows no drafts for these functions. Repeated at finish (see
`validation.md`).

## Relied-upon claims

All present in the imported `provenance.yaml` files.

| claim_id | category | normative | used by |
|---|---|---|---|
| fplot.signature-primary | black-box-observation | yes | FR-065-01 |
| fplot.object | black-box-observation | yes | FR-065-01 (D-065-1) |
| fplot.default-interval | black-box-observation | yes | FR-065-02 |
| fplot.interval | black-box-observation | yes | FR-065-03 |
| fplot.cobra-form | black-box-observation | yes | FR-065-04 |
| fplot.linespec | black-box-observation | yes | FR-065-05 |
| fplot.ax | black-box-observation | yes | FR-065-06 |
| fplot.numeric | black-box-observation | **no** | FR-065-09 (implemented as observed) |
| fplot.default-color | black-box-observation | **no** | FR-065-10 (not asserted) |
| fplot.default-color-dependence | inference, unresolved | **no** | FR-065-10 (documented choice) |
| fplot.error-decreasing | black-box-observation | yes | FR-065-08 |
| fplot.error-bad-property | black-box-observation | yes | FR-065-08 |
| fplot.warning-char-fn | black-box-observation | yes | FR-065-07 (D-065-3) |
| rectangle.signature-primary | black-box-observation | yes | FR-065-20 (D-065-4) |
| rectangle.defaults | black-box-observation | yes | FR-065-21 |
| rectangle.cobra-form | black-box-observation | yes | FR-065-22 |
| rectangle.curvature | black-box-observation | yes | FR-065-23 |
| rectangle.colors | black-box-observation | yes | FR-065-24 |
| rectangle.parent | black-box-observation | yes | FR-065-25 (D-065-5) |
| rectangle.default-edge-color | black-box-observation | **no** | FR-065-21 (documented choice) |
| rectangle.default-edge-color-dependence | inference, unresolved | **no** | FR-065-21 |
| rectangle.error-negative-width | black-box-observation | yes | FR-065-26 |
| rectangle.error-position-length | black-box-observation | yes | FR-065-26 |
| rectangle.error-curvature-range | black-box-observation | yes | FR-065-26 |
| rectangle.error-bad-property | black-box-observation | yes | FR-065-26 |
| validatecolor.signature-primary | black-box-observation | yes | FR-065-30 |
| validatecolor.cobra-form | black-box-observation | yes | FR-065-30 |
| validatecolor.names | black-box-observation | yes | FR-065-31 |
| validatecolor.triplets | black-box-observation | yes | FR-065-32 |
| validatecolor.hex | black-box-observation | yes | FR-065-33 |
| validatecolor.multiple | black-box-observation | yes | FR-065-34 |
| validatecolor.error-matrix-without-multiple | black-box-observation | yes | FR-065-35 |
| validatecolor.error-unknown-name | black-box-observation | yes | FR-065-35 |
| validatecolor.error-out-of-range | black-box-observation | yes | FR-065-35 |
| validatecolor.error-two-components | black-box-observation | yes | FR-065-35 |
| validatecolor.error-bad-option | black-box-observation | yes | FR-065-35 |

Backing observations (imported): `fplot_observations.json` (12 cases),
`rectangle_observations.json` (12), `validatecolor_observations.json` (21).

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| fplot.q-warning-coverage, rectangle.q-, validatecolor.q-warning-coverage | only the observed char-function warning is raised |
| fplot.q-theme | default colour = axes colour order (not asserted); default LineWidth 0.5 asserted because `fplot.linespec` states it normatively |
| fplot.q-points | 201 fixed points (FR-065-10, D-065-2) |
| rectangle.q-theme | default EdgeColor `[0 0 0]` (documented choice); LineWidth 0.5 asserted (normative `rectangle.defaults`) |
| rectangle.q-error-ids | `MATLAB:hg:…` identifiers reproduced as `RunMat:hg:…`; flagged for human review |
| validatecolor.q-error-ids | `RunMat:unrecognizedStringChoice` reproduced; flagged for human review |
| validatecolor.q-names | RunMat's eight-name plotting colour table (FR-065-31) |

## Source categories and independence

- Only the claims marked normative above are requirements. Observed
  `MATLAB:x:y` identifiers are raised as `RunMat:x:y`.
- The sampling grid, vectorised-then-pointwise evaluation, rounded-corner
  geometry (corner radii from Curvature), the colour table, uint16
  scaling and all error identifiers for unobserved conditions are
  **independently derived documented choices**, tagged in tests
  (`unresolved_choice_*`, `deviation_*`) and not asserted as
  MATLAB-conformant.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository.
