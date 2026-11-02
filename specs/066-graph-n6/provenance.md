# Provenance Record — 066-graph-n6

| Field | `subgraph` | `findnode` |
|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same |
| Approved specification | `subgraph` **0.2.0**, `status: approved` | `findnode` **0.2.0**, approved |
| Specification revision (import pin) | **`e364206`** (IMPORT-MANIFEST Batch 15, N6) | same |
| Review status | approved 2026-11-02 (source `docs/review/2026-11-02-n6-new-specs-human-loop.md`) | same |
| Observations | 20 cases | 25 cases |
| Import date | 2026-11-02 (imported copies under `specs/imports/matlab-interface-spec/specifications/`) | same |
| Selection | maintainer, 2026-11-02, `specs/001-clean-room-matlab-builtins/selection-proposal-n6.md` (F66; G1–G3 pre-approved) | same |
| Applicable release | R2026a (as recorded in each imported `provenance.yaml`) | same |

## Freshness check (start)

2026-11-02, in `../matlab-interface-spec` after `git fetch -q origin`
(rc 0): `HEAD` = `origin/main` = `20825f8`, one commit after the pin
`e364206` ("Add COBRA core-MATLAB builtin dependency list (CSV)").
`git log e364206..origin/main -- specifications/{subgraph,findnode}
experiments/{subgraph,findnode}` is empty; `git status --short` for those
directories is empty (no drafts); `diff -rq` of the imported copies against
the source tree reports no difference. The imported 0.2.0 copies are
current. Final check: `validation.md`.

## Relied-upon claims

All ids below verified present in the imported 0.2.0 `provenance.yaml`
files (`source_category: black-box-observation`, `status: confirmed`,
`normative: true`).

| claim_id | used by |
|---|---|
| subgraph.signature-primary | FR-066-01 |
| subgraph.output-class | FR-066-01 |
| subgraph.node-order | FR-066-02 |
| subgraph.node-table | FR-066-03 |
| subgraph.induced-edges | FR-066-04 |
| subgraph.edge-order | FR-066-05 |
| subgraph.multigraph | FR-066-06 |
| subgraph.isolated | FR-066-06 |
| subgraph.empty | FR-066-07 |
| subgraph.names | FR-066-08 |
| subgraph.error-repeat | FR-066-09 |
| subgraph.error-range | FR-066-09 |
| subgraph.error-unknown-name | FR-066-09 |
| subgraph.error-one-arg | FR-066-10 |
| findnode.signature-primary | FR-066-11 |
| findnode.output-class | FR-066-11 |
| findnode.cobra-form | FR-066-12 |
| findnode.names | FR-066-12 |
| findnode.not-found | FR-066-13 |
| findnode.numeric | FR-066-14 |
| findnode.error-unnamed-char | FR-066-15 |
| findnode.error-logical | FR-066-16 |
| findnode.error-numeric-zero | FR-066-16 |
| findnode.error-one-arg | FR-066-17 |

Error identifiers: the exports record identifiers in the MATLAB namespace.
RunMat raises the final component in its own per-function namespace (same
mapping as features 036, 043, 059 and 062; maintainer decision
2026-11-02):

| observed (export) | raised by RunMat |
|---|---|
| `MATLAB:graphfun:subgraph:InvalidInd` | `RunMat:subgraph:InvalidInd` |
| `MATLAB:graphfun:digraph:InvalidNodeID` (from subgraph) | `RunMat:subgraph:InvalidNodeID` |
| `MATLAB:graphfun:digraph:UnknownNodeName` (from subgraph) | `RunMat:subgraph:UnknownNodeName` |
| `MATLAB:graphfun:findnode:NoNames` | `RunMat:findnode:NoNames` |
| `MATLAB:graphfun:findnode:ArgType` | `RunMat:findnode:ArgType` |
| `MATLAB:graphfun:findnode:PosInt` | `RunMat:findnode:PosInt` |
| `MATLAB:minrhs` (both) | `RunMat:minrhs` |

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| subgraph.q-warning-coverage | implementation emits no warnings |
| subgraph.q-tie-order | U-066-5: stable (stored order) |
| subgraph.q-undirected-ids | U-066-4: same identifiers for graph inputs |
| findnode.q-warning-coverage | implementation emits no warnings |
| findnode.q-query-types | U-066-9 |

Local documented choices U-066-1 … U-066-9 and deviations D-066-1, D-066-2
are in `spec.md`.

## Source categories and independence

- Only the claims above are relied upon (all 24 claims of the two exports,
  all normative).
- Algorithms are elementary: for `subgraph`, node selection, an
  old→new index map, a filter of the edge list, renumbering, a stable sort
  by end nodes and row selection of the table variables (the same steps as
  F62's `reordernodes`); for `findnode`, an exact name lookup or an
  identity map on validated IDs.
- No external implementation code was used (no licence to review).
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review MAY
  still be required.
