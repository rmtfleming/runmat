# Provenance Record — 034-table-variable-editing

| Field | `addvars` | `removevars` | `movevars` |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `addvars` **0.3.0**, `status: approved` | `removevars` **0.3.0**, `status: approved` | `movevars` **0.3.0**, `status: approved` |
| Specification revision (import pin) | **`5d595da9c78949bcf1470384a62111bffb215097`** ("Widen and re-approve Tier B table specs (W1)"; IMPORT-MANIFEST Batch 7) | same | same |
| Review status | approved 2026-10-29 after human-loop review (source `docs/review/2026-10-29-tier-b-w1-widening-human-loop.md`, "Approve all 4") | same | same |
| Import date | 2026-10-29 | 2026-10-29 | 2026-10-29 |
| Freshness check | 2026-10-29, at start and at finish: `git diff --name-only 5d595da HEAD` and `… 5d595da origin/main` over the three `specifications/<fn>/` directories are empty (source HEAD = origin/main: `8614c5b` at start, `c65e238` at finish), and `git status --short` over them is clean | same | same |
| Applicable release | as recorded in the imported `interface.yaml` / `provenance.yaml` | same | same |

Previous pins (0.2.0, batch C1): `addvars`, `removevars` `e46587a`
(IMPORT-MANIFEST Batch 3); `movevars` `4445294` (Batch 4). They are
superseded by Batch 7.

## Relied-upon claims (0.3.0)

Every id below was checked against the imported
`specs/imports/matlab-interface-spec/specifications/<fn>/provenance.yaml`.
All are `source_category: black-box-observation`, `status: confirmed`.

| claim_id | normative | used by |
|---|---|---|
| addvars.signature-primary | yes | FR-034-01 |
| addvars.output-class | yes | FR-034-01 |
| addvars.appends-variable | yes | FR-034-01 |
| addvars.multiple-variables | yes | FR-034-01 |
| addvars.data-preserved | yes | FR-034-01 |
| addvars.before | yes | FR-034-02 |
| addvars.after | yes | FR-034-02 |
| addvars.new-names | yes | FR-034-03 |
| addvars.default-name-position | yes | FR-034-04 |
| addvars.default-name-input | yes | FR-034-04 |
| addvars.before-past-end | **no** | FR-034-12 (not implemented; see q-before-past-end) |
| addvars.error-before-and-after | yes | FR-034-05 |
| addvars.error-rows | yes | FR-034-05 |
| addvars.error-dup-name | yes | FR-034-05 |
| addvars.error-names-count | yes | FR-034-05 |
| addvars.error-bad-location | yes | FR-034-05 |
| addvars.error-index-range | yes | FR-034-05 |
| removevars.signature-primary | yes | FR-034-06 |
| removevars.output-class | yes | FR-034-06 |
| removevars.removes-variable | yes | FR-034-06 |
| removevars.by-index | yes | FR-034-06 |
| removevars.by-logical | yes | FR-034-06 |
| removevars.multiple-names | yes | FR-034-06 |
| removevars.empty-selection | yes | FR-034-06 |
| removevars.remove-all | yes | FR-034-10 (**deviation D-034-1**) |
| removevars.short-logical-mask | **no** | FR-034-12 (not implemented; see q-short-mask) |
| removevars.error-unknown | yes | FR-034-07 |
| removevars.error-index-range | yes | FR-034-07 |
| removevars.error-index-zero | yes | FR-034-07 |
| movevars.signature-primary | yes | FR-034-08 |
| movevars.output-class | yes | FR-034-08 |
| movevars.moves-variable | yes | FR-034-08 |
| movevars.after | yes | FR-034-08 |
| movevars.numeric-location | yes | FR-034-08 |
| movevars.numeric-vars | yes | FR-034-08 |
| movevars.multiple-order | yes | FR-034-08 |
| movevars.self-location | yes | FR-034-08 |
| movevars.default-end | yes | FR-034-08 |
| movevars.contents-follow | yes | FR-034-08 |
| movevars.error-before-and-after | yes | FR-034-09 |
| movevars.error-unknown-var | yes | FR-034-09 |
| movevars.error-unknown-location | yes | FR-034-09 |
| movevars.error-index-range | yes | FR-034-09 |

Backing observations: the imported `observations/<fn>_observations.json`
(addvars 25 cases, removevars 19, movevars 18). Every non-erroring case
except `removevars` `all` (D-034-1) and the two non-normative cases
(`before-past-end`, `logical-short`) is reproduced by a normative test (see
`validation.md`).

Error identifiers: the claims confirm `MATLAB:`-namespaced identifiers.
RunMat raises them in its own error namespace (`RunMat:<rest>`, FR-034-11),
the mapping `runtime_error::semantic_error` already applies to
`MATLAB:`-style identifiers.

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| addvars.q-default-name-collision | documented choice: colliding default made unique (FR-034-13) |
| addvars.q-before-past-end | `RunMat:addvars:UnsupportedOption` (FR-034-12) |
| addvars.q-input-name-scope | documented choice: plain identifiers only; otherwise the position name (FR-034-14) |
| removevars.q-logical-longer | `RunMat:removevars:UnsupportedOption` (FR-034-12) |
| removevars.q-short-mask | `RunMat:removevars:UnsupportedOption` (FR-034-12) |
| movevars.q-location-in-vars | `RunMat:movevars:UnsupportedOption` (FR-034-12) |

The 0.2.0 questions `addvars.q-position`, `addvars.q-names`,
`removevars.q-index`, `removevars.q-multiple`, `movevars.q-after` and
`movevars.q-index` are resolved in 0.3.0 and implemented. Their former
`UnsupportedOption` disposition is superseded (see spec.md, "Superseded
0.2.0 dispositions").

## Source categories and independence

- Normative behaviour comes only from the claims above that are marked
  "yes".
- Independently derived documented choices (FR-034-11 identifier namespace,
  FR-034-13..16) rest on RunMat's own conventions: the table naming helpers,
  the VM's builtin callsite record (as used by `display`), the
  `semantic_error` namespace mapping, and the table subsystem's metadata
  model. Tests tag them `unresolved_choice_*`, `rejects_*` or `deviation_*`
  and do not assert them as MATLAB-conformant.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. The imported observations'
  `call` strings were read only as test inputs.
