# Provenance Record — 048-containers-map-conformance

| Field | `keys` | `isKey` | `remove` |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `keys` **0.2.0**, `status: approved` | `isKey` **0.2.0**, approved | `remove` **0.2.0**, approved |
| Specification revision (import pin) | **`8f72edb119511f95884fb63f550e6c88682c327b`** (IMPORT-MANIFEST Batch 10, N2) | same | same |
| Review status | approved 2026-10-30 (source `docs/review/2026-10-30-n2-new-specs-human-loop.md`) | same | same |
| Import date | 2026-10-30 (imported copies under `specs/imports/matlab-interface-spec/specifications/`) | same | same |
| Freshness check | 2026-10-30, at start and at finish: after `git fetch -q origin`, `git log 8f72edb..HEAD` and `8f72edb..origin/main` over the three spec dirs were empty; `git status --short` for them was clean (see validation.md) | same | same |
| Applicable release | R2026a, as recorded in the imported `provenance.yaml` | same | same |

## Relied-upon claims

All ids verified present in the imported 0.2.0 `provenance.yaml` files
(`status: confirmed`, `source_category: black-box-observation`). Normative
unless marked.

| claim_id | used by |
|---|---|
| keys.signature-primary | FR-048-01 |
| keys.output-class | FR-048-01 |
| keys.sorted | FR-048-02 |
| keys.aligned-with-values | FR-048-03 |
| keys.method-syntax | FR-048-04 |
| keys.non-map (**non-normative**) | U-048-1 |
| isKey.signature-primary | FR-048-05 |
| isKey.output-class | FR-048-05 |
| isKey.membership | FR-048-05 |
| isKey.cell-query | FR-048-06 |
| isKey.numeric | FR-048-07 |
| isKey.lenient | FR-048-08 |
| isKey.string-key | FR-048-09 |
| remove.signature-primary | FR-048-10 |
| remove.removes | FR-048-10 |
| remove.handle | FR-048-11 |
| remove.returns-map | FR-048-11 |
| remove.present-removed-with-absent | FR-048-12 |
| remove.warning-absent | FR-048-12 |

Warning identifiers: the export records `MATLAB:Containers:Map:NoKeyToRemove`;
per the programme rule RunMat issues `RunMat:Containers:Map:NoKeyToRemove`.

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| keys.q-warning-coverage | no warnings emitted |
| isKey.q-warning-coverage | no warnings emitted |
| remove.q-warning-coverage | only the FR-048-12 warning is emitted |

## Source categories and independence

- Only the claims above are relied upon (18 normative, 1 non-normative).
- The code under test is upstream RunMat code; the fixes are local changes to
  its own control flow (membership on class mismatch, warning instead of
  error, row-major/column-major alignment of the `isKey` result). Nothing is
  derived from MATLAB internals.
- U-048-1 … U-048-3 are choices, tagged `unresolved_choice_*` in tests and
  not asserted as MATLAB behaviour.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review MAY
  still be required.
