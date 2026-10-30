# Provenance Record — 038-validateattributes

| Field | `validateattributes` |
|---|---|
| Source repository | `matlab-interface-spec` (local) |
| Approved specification | `validateattributes` **0.3.0**, `status: approved` |
| Specification revision (import pin) | **`8614c5b396851cdac1d1eb58f1cf5f7fa7f8909e`** (IMPORT-MANIFEST Batch 8, Tier B widening W2) |
| Review status | approved 2026-10-29 (source `docs/review/2026-10-29-tier-b-w2-widening-human-loop.md`, "Approve all 7") |
| Import date | 2026-10-29 |
| Imported files | `specs/imports/matlab-interface-spec/specifications/validateattributes/` (`interface.yaml`, `behaviour.md`, `provenance.yaml`, `observations/validateattributes_observations.json`, 58 cases) |
| Freshness check | 2026-10-29: in `../matlab-interface-spec`, after `git fetch -q --all`, `HEAD` = `origin/main` = `8614c5b`; `git diff --name-only 8614c5b HEAD -- specifications/validateattributes` and the same against `origin/main` were empty; `git status --short` for that path was empty. Nothing newer or in progress. |
| Superseded | 0.2.0 (pin `104fb5f`, approval at `b9387dc`, Batch 6): the original basis of this feature, replaced by 0.3.0 before any code was compiled. |
| Applicable release | as recorded in the import (see the imported `provenance.yaml`) |

## Relied-upon claims

All 43 claim ids in the imported `provenance.yaml` were checked against the
FR citations in `spec.md` (script in `validation.md`). Every claim is
`source_category: black-box-observation`, `normative: true`.

| claim_id | used by |
|---|---|
| validateattributes.signature-primary | FR-038-01 |
| validateattributes.pass-silent | FR-038-01 |
| validateattributes.class-list | FR-038-02 |
| validateattributes.empty-attributes | FR-038-03 |
| validateattributes.attribute-semantics | FR-038-05 |
| validateattributes.each-attribute-checked | FR-038-04 |
| validateattributes.funcname-identifier | FR-038-07 (identifier part non-normative per decision (b)) |
| validateattributes.error-fail-type | FR-038-02 |
| validateattributes.error-empty-class-list | FR-038-02 |
| validateattributes.error-fail-scalar, -vector, -row, -column, -2d, -nonempty, -square, -positive, -nonnegative, -integer, -finite, -nonnan, -real, -nonzero, -nonsparse, -binary, -even, -odd, -increasing, -decreasing, -nonincreasing, -gt, -ge, -lt, -le, -numel, -size, -ncols, -nrows, -ndims (30 ids, each `validateattributes.error-fail-<name>`) | FR-038-04, FR-038-05 |
| validateattributes.error-returns-nothing | FR-038-06 |
| validateattributes.error-fail-extra-args | FR-038-07 |
| validateattributes.error-fail-extra-varname | FR-038-07 |
| validateattributes.warning-unknown-attribute | FR-038-08 |

Decision (b) (D2 = B retained): the identifiers recorded in these claims
are used only to choose the RunMat suffix (the last component). The
`MATLAB:` prefix and the `funcName` middle component are non-normative and
are never emitted.

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| validateattributes.q-check-order | documented choice: first failure in list order |
| validateattributes.q-more-attributes | `UnsupportedOption` (non-numeric value attributes, `'diag'`, `'scalartext'`); `logical` and `'decreasing'` pass semantics are documented choices |
| q-warning-coverage (IMPORT-MANIFEST Batch 8) | only the confirmed `attributeNotFound` warning is emitted |

Local items U-038-1…8 are listed in `spec.md`.

## Source categories and independence

- Normative: the 43 claims above (with identifiers mapped per decision (b)).
- Independently derived documented choices: the precise shape and value
  definitions (FR-038-09..12), the `'numeric'` category membership beyond
  `double`, `logical` value semantics, string-array class lists, check
  order, argument-form and value-form errors, `expectedNondecreasing`, all
  message text. They are derived from RunMat's own `class()`/`size()`
  semantics and are not asserted as MATLAB-conformant. Tests for them are
  named `documented_choice_*`, `unresolved_choice_*` or `rejects_*`.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. No external implementation
  source was used. Independent legal review MAY still be required.
