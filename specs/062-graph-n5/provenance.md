# Provenance Record — 062-graph-n5

| Field | `outedges` | `reordernodes` |
|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same |
| Approved specification | `outedges` **0.2.0**, `status: approved` | `reordernodes` **0.2.0**, approved |
| Specification revision (import pin) | **`1c91aa1`** (IMPORT-MANIFEST Batch 14, N5) | same |
| Review status | approved 2026-10-30 (source `docs/review/2026-10-30-n5-new-specs-human-loop.md`) | same |
| Observations | 14 cases | 11 cases |
| Import date | 2026-10-31 (imported copies under `specs/imports/matlab-interface-spec/specifications/`) | same |
| Selection | maintainer, 2026-10-31, `specs/001-clean-room-matlab-builtins/selection-proposal-n5.md` (F62; G1–G3 pre-approved) | same |
| Applicable release | R2026a (as recorded in each imported `provenance.yaml`) | same |

## Freshness check (start)

2026-10-31, in `../matlab-interface-spec` after `git fetch -q origin`
(rc 0): `HEAD` = `origin/main` = `1c91aa1` (the pin).
`git log 1c91aa1..origin/main -- specifications/{outedges,reordernodes}`
is empty; `git status --short` shows no change under those
`specifications/` directories or their `experiments/` folders (tracked at
the pin); `diff -r` of the imported copies against the source tree reports
no difference. Final check: `validation.md`.

## Relied-upon claims

All ids below verified present in the imported 0.2.0 `provenance.yaml`
files (`source_category: black-box-observation`, `status: confirmed`,
`normative: true`).

| claim_id | used by |
|---|---|
| outedges.signature-primary | FR-062-01 |
| outedges.order | FR-062-01, FR-062-02 |
| outedges.empty | FR-062-03 |
| outedges.names | FR-062-04 |
| outedges.multi | FR-062-05 |
| outedges.undirected | FR-062-06 |
| outedges.error-range | FR-062-07 |
| outedges.error-vector | FR-062-07 |
| outedges.error-unknown-name | FR-062-07 |
| reordernodes.signature-primary | FR-062-08 |
| reordernodes.nodes | FR-062-09 |
| reordernodes.edges | FR-062-10, FR-062-11 |
| reordernodes.weights | FR-062-10, FR-062-11 |
| reordernodes.undirected | FR-062-10 |
| reordernodes.names | FR-062-09, FR-062-12 |
| reordernodes.identity | FR-062-08 |
| reordernodes.error-repeat | FR-062-13 |
| reordernodes.error-range | FR-062-13 |

Error identifiers: the exports record identifiers in the MATLAB namespace.
RunMat raises the final component in its own per-function namespace (same
mapping as features 036, 043 and 059):

| observed (export) | raised by RunMat |
|---|---|
| `…:digraph:InvalidNodeID` (from outedges) | `RunMat:outedges:InvalidNodeID` |
| `…:graphbuiltin:InvalidNodeScalar` (from outedges) | `RunMat:outedges:InvalidNodeScalar` |
| `…:digraph:UnknownNodeName` (from outedges) | `RunMat:outedges:UnknownNodeName` |
| `…:reordernodes:InvalidOrder` | `RunMat:reordernodes:InvalidOrder` |
| `…:digraph:InvalidNodeID` (from reordernodes) | `RunMat:reordernodes:InvalidNodeID` |

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| outedges.q-warning-coverage | implementation emits no warnings |
| outedges.q-undirected-ids | U-062-4: graph inputs use the same identifiers |
| reordernodes.q-warning-coverage | implementation emits no warnings |

Local documented choices U-062-1 … U-062-9 and deviations D-062-1, D-062-2
(and D-036-2) are in `spec.md`.

## Source categories and independence

- Only the claims above are relied upon (all 18 claims of the two exports,
  all normative).
- Algorithms are elementary: a filter over the stored edge list for
  `outedges`; for `reordernodes`, an inverse permutation, renumbering of
  each edge, a stable sort by end nodes and row selection of the table
  variables.
- No external implementation code was used (no licence to review).
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review MAY
  still be required.
