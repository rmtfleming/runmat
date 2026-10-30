# Provenance Record — 045-text-conversion

| Field | `regexptranslate` | `dec2base` | `validatestring` |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `regexptranslate` **0.2.0**, `status: approved` | `dec2base` **0.2.0**, `status: approved` | `validatestring` **0.2.0**, `status: approved` |
| Observations | 13 cases | 17 cases | 13 cases |

| Field | Value (all three) |
|---|---|
| Specification revision (import pin) | **`c65e2382fa2420416439f8342333af77dcaee5b3`** (IMPORT-MANIFEST Batch 9, N1) |
| Review status | approved 2026-10-29 (source `docs/review/2026-10-29-n1-new-specs-human-loop.md`, "Approve all 14") |
| Import date | 2026-10-29 |
| Selection | maintainer, 2026-10-30, `specs/001-clean-room-matlab-builtins/selection-proposal-n1.md` (F45; G1–G3 pre-approved) |
| Freshness check (start) | 2026-10-30: in `../matlab-interface-spec`, after `git fetch -q --all`, HEAD = origin/main = `8f72edb`. `git diff --name-only c65e238 HEAD` and the same against `origin/main`, both over `specifications/{regexptranslate,dec2base,validatestring}`, returned nothing; `git status --short` for those paths was clean. |
| Freshness check (finish) | recorded in `validation.md` |
| Applicable release | as recorded in each imported `provenance.yaml` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/{regexptranslate,dec2base,validatestring}/{interface.yaml,behaviour.md,provenance.yaml,observations/*_observations.json}` |

## Relied-upon claims

All are `source_category: black-box-observation`, `normative: true`. Every id
below was checked against the imported `provenance.yaml` (script output in
`validation.md`).

| claim_id | used by |
|---|---|
| regexptranslate.signature-primary | FR-045-01 |
| regexptranslate.escape | FR-045-02 |
| regexptranslate.escape-whitespace | FR-045-02 |
| regexptranslate.wildcard | FR-045-03 |
| regexptranslate.wildcard-use | FR-045-03 |
| regexptranslate.types | FR-045-04 |
| regexptranslate.error-op | FR-045-05 |
| dec2base.signature-primary | FR-045-06 |
| dec2base.output-class | FR-045-06 |
| dec2base.digits | FR-045-07 |
| dec2base.zero | FR-045-07 |
| dec2base.padding | FR-045-08 |
| dec2base.vector | FR-045-09 |
| dec2base.integer-class | FR-045-10 |
| dec2base.large | FR-045-10 |
| dec2base.error-negative | FR-045-11 |
| dec2base.error-base1 | FR-045-11 |
| validatestring.signature-primary | FR-045-12 |
| validatestring.exact | FR-045-13 |
| validatestring.prefix | FR-045-13 |
| validatestring.case-insensitive | FR-045-13 |
| validatestring.nested-options | FR-045-14 |
| validatestring.error-ambiguous | FR-045-14 |
| validatestring.output-type | FR-045-15 |
| validatestring.error-nomatch | FR-045-16 |
| validatestring.error-funcname | FR-045-12, FR-045-16 (identifier part mapped per D-045-1) |
| validatestring.error-cell-of-strings | FR-045-17 |

## Unresolved questions carried

`regexptranslate.q-warning-coverage`, `regexptranslate.q-other-ops`,
`dec2base.q-warning-coverage`, `dec2base.q-large-values`,
`dec2base.q-char-base` (flagged by the maintainer), and
`validatestring.q-warning-coverage`, `validatestring.q-more-args`.
`spec.md` ("Unresolved behaviour") records how each is handled.

## Source categories and independence

- FR-045-01..17 come from black-box observations. The observed identifiers
  are in the `MATLAB:` namespace, and RunMat raises them in its own
  namespace (D-045-1).
- FR-045-18..28 are **independently derived documented choices** or
  explicit `UnsupportedOption` dispositions. Tests tag them
  `documented_choice_*` / `unresolved_*`.
- Algorithms, all written for this feature:
  - regexptranslate: a per-character substitution table.
  - dec2base: textbook repeated division on `u64`.
  - validatestring: a case-folded prefix filter, followed by a check for a
    match that is a prefix of every other match.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
