# Provenance Record — 052-data-n2

| Field | `addnode` | `categories` | `textread` |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `addnode` **0.2.0**, `status: approved` | `categories` **0.2.0**, approved | `textread` **0.2.0**, approved |
| Specification revision (import pin) | **`8f72edb119511f95884fb63f550e6c88682c327b`** (IMPORT-MANIFEST Batch 10, N2) | same | same |
| Review status | approved 2026-10-30 (source `docs/review/2026-10-30-n2-new-specs-human-loop.md`) | same | same |
| Import date | 2026-10-30 (imported copies under `specs/imports/matlab-interface-spec/specifications/`) | same | same |
| Freshness check | 2026-10-30, before starting and before finishing (see validation.md) | same | same |
| Applicable release | R2026a, as recorded in the imported `provenance.yaml` | same | same |

## Relied-upon claims

All ids below verified present in the imported 0.2.0 `provenance.yaml`
files (`status: confirmed`, `source_category: black-box-observation`).
Normative unless marked.

| claim_id | used by |
|---|---|
| addnode.signature-primary | FR-052-01 |
| addnode.count | FR-052-01 |
| addnode.class | FR-052-01 |
| addnode.names | FR-052-02 |
| addnode.names-to-unnamed | FR-052-03 |
| addnode.error-dup-name | FR-052-04 |
| addnode.error-negative | FR-052-04 |
| categories.signature-primary | FR-052-05 |
| categories.output-class | FR-052-05 |
| categories.sorted-default | FR-052-06 |
| categories.explicit-order | FR-052-06 |
| categories.numeric | FR-052-06 |
| categories.missing | FR-052-07 |
| categories.empty | FR-052-07 |
| categories.non-categorical (**non-normative**) | U-052-3 (documented choice) |
| textread.signature-primary | FR-052-08 |
| textread.columns | FR-052-08 |
| textread.types | FR-052-09 |
| textread.options | FR-052-10 |
| textread.skip | FR-052-11 |
| textread.default-format | FR-052-12 |
| textread.empty | FR-052-13 |
| textread.no-warning | FR-052-14 |
| textread.error-missing | FR-052-15 |

Error identifiers: the exports record identifiers in the MATLAB namespace.
RunMat raises the final component in its own per-function namespace (same
mapping as features 036/043):

| observed (export) | raised by RunMat |
|---|---|
| `…:graph:NonUniqueNames` (from addnode) | `RunMat:addnode:NonUniqueNames` |
| `…:addnode:InvalidNrNodes` | `RunMat:addnode:InvalidNrNodes` |
| `…:textread:FileNotFound` | `RunMat:textread:FileNotFound` |
| `UndefinedFunction` (categories, non-normative) | `RunMat:categories:UndefinedFunction` |

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| addnode.q-warning-coverage | implementation emits no warnings |
| addnode.q-graph-variables | other node inputs (tables etc.) raise UnsupportedOption (U-052-2) |
| categories.q-warning-coverage | implementation emits no warnings |
| textread.q-more-formats | only `%f`, `%d`, `%s`, `%*s`, `headerlines`, `delimiter`; others UnsupportedOption (U-052-4, U-052-6) |

Local documented choices U-052-1 … U-052-9 are in `spec.md`.

## Source categories and independence

- Only the claims above are relied upon; 23 are normative, 1 is
  non-normative and backs a documented choice only.
- `addnode` is list manipulation on upstream's `GraphData`; `categories`
  reads the `Categories` property of upstream's categorical object;
  `textread` is an independent whitespace/delimiter tokenizer written for
  this feature (upstream `textscan` helpers are private to its module and
  file-handle oriented, so they were not reused).
- Every U-052-n item is a choice, tagged `unresolved_choice_*` in tests and
  not asserted as MATLAB behaviour.
- No external implementation code was used (no licence to review).
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review MAY
  still be required.
