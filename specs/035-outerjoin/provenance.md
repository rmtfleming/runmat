# Provenance Record — 035-outerjoin

| Field | `outerjoin` |
|---|---|
| Source repository | `matlab-interface-spec` (local) |
| Approved specification | `outerjoin` 0.3.0, `status: approved` |
| Specification revision (import pin) | **`5d595da9c78949bcf1470384a62111bffb215097`** ("Widen and re-approve Tier B table specs (W1)"), IMPORT-MANIFEST Batch 7 |
| Review status | approved 2026-10-29 after human-loop review (source `docs/review/2026-10-29-tier-b-w1-widening-human-loop.md`) |
| Import date | 2026-10-29 |
| Freshness check | 2026-10-29, at start and at finish: `git fetch`, then `git diff --name-only 5d595da HEAD -- specifications/outerjoin`, the same against `origin/main`, and `git status --short -- specifications/outerjoin` were all empty. The imported files are byte-identical to the source (`diff -q`). |
| Applicable release | as recorded in the imported `interface.yaml` / `provenance.yaml` |
| Superseded | 0.2.0 import (Batch 4), under which F35 was deferred (B-035-1) |

## Relied-upon claims (all `source_category: black-box-observation`, `normative: true`, `status: confirmed`)

Each id below is verified against the imported
`specs/imports/matlab-interface-spec/specifications/outerjoin/provenance.yaml`.

| claim_id | used by | backing cases |
|---|---|---|
| outerjoin.signature-primary | FR-035-01 | basic |
| outerjoin.output-class | FR-035-01 | basic, default-full, mergekeys, type-left, fill-types |
| outerjoin.table-shape | FR-035-01 | basic |
| outerjoin.key-inference | FR-035-02 | default-full, multi-key |
| outerjoin.keys-option | FR-035-03 | keys-name, keys-index, leftright-keys |
| outerjoin.type-rows | FR-035-04 | default-full, type-full, type-left, type-right, type-inner |
| outerjoin.row-order | FR-035-05 | unsorted, unsorted-merge, default-full, multi-key, multi-key-unsorted |
| outerjoin.duplicate-keys | FR-035-06 | dup-keys, dup-keys-desc |
| outerjoin.column-order | FR-035-07 | default-full, type-left, type-right, mergekeys, multi-key, leftright-keys-merge, key-second-column, key-second-column-nomerge |
| outerjoin.mergekeys | FR-035-08 | mergekeys, type-left-merge, unsorted-merge, leftright-keys-merge |
| outerjoin.name-suffix | FR-035-09 | default-full, unsorted, same-nonkey-name, basic, leftright-keys, key-second-column-nomerge |
| outerjoin.fill-values | FR-035-10 | default-full, type-left, fill-types |
| outerjoin.inner-key-single | FR-035-11 | type-inner |
| outerjoin.error-no-common | FR-035-02, FR-035-12 | err-no-common |
| outerjoin.error-bad-type | FR-035-04, FR-035-12 | err-bad-type |
| outerjoin.error-keys-and-leftkeys | FR-035-03, FR-035-12 | err-keys-and-leftkeys |

`outerjoin.signature-primary` is recorded in the export as a black-box
observation of the confirmed primary form. The other four invocation forms
(`Keys`, `LeftKeys`/`RightKeys`, `MergeKeys`, `Type`) are
`status: confirmed` public-interface facts in `interface.yaml`, backed by
the cases cited above.

All 25 observation cases are reproduced as tests. The error cases record
only an identifier; RunMat maps each to its own `RunMat:outerjoin:<Id>`
identifier (FR-035-12) and does not reproduce message text.

## Unresolved questions carried (returned to specification side)

| id | disposition |
|---|---|
| outerjoin.q-index-outputs | `RunMat:outerjoin:UnsupportedOption` |
| outerjoin.q-select-variables | `RunMat:outerjoin:UnsupportedOption` |
| outerjoin.q-many-to-many | `RunMat:outerjoin:UnsupportedOption` |
| outerjoin.q-key-types | `RunMat:outerjoin:UnsupportedOption` |
| outerjoin.q-name-suffix-context | documented choice (FR-035-17); flagged for human review |

Local items U-035-1…9 are listed in `spec.md`.

## Source categories and independence

- Normative: the 16 claims above (black-box observations) and the
  confirmed invocation forms (public-interface facts).
- Independently derived: the join algorithm, a standard relational equi-join.
  RunMat builds an ordered map from right key tuples to right rows, emits
  matched and kept rows, and stable-sorts by key tuple. It also covers every
  FR-035-13..22 disposition. None of these is presented as an observation.
  They are tagged in tests (`unresolved_*`, `documented_choice_*`,
  `rejects_*`).
- The caller-name mechanism (FR-035-17) reuses RunMat's existing call-site
  recording (`crate::callsite`), which `display` and `inputname` also use.
  It is RunMat architecture, not derived from MATLAB.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. No external implementation
  code was used, so there is no third-party licence to record. Independent
  legal review MAY still be required.
