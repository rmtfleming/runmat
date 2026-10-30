# Provenance Record — 043-graph-editing

| Field | `rmnode` | `addedge` | `rmedge` | `laplacian` |
|---|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same | same |
| Approved specification | `rmnode` **0.2.0**, `status: approved` | `addedge` **0.2.0**, approved | `rmedge` **0.2.0**, approved | `laplacian` **0.2.0**, approved |
| Specification revision (import pin) | **`c65e2382fa2420416439f8342333af77dcaee5b3`** (IMPORT-MANIFEST Batch 9, N1) | same | same | same |
| Review status | approved 2026-10-29 (source `docs/review/2026-10-29-n1-new-specs-human-loop.md`) | same | same | same |
| Import date | 2026-10-30 (imported copies under `specs/imports/matlab-interface-spec/specifications/`) | same | same | same |
| Freshness check | 2026-10-30, before starting and before finishing: after `git fetch -q --all`, `git diff --name-only c65e238 HEAD` and `… c65e238 origin/main` over the four spec dirs were empty; `git status --short` for them was clean (see validation.md) | same | same | same |
| Applicable release | as recorded in the imported `provenance.yaml` | same | same | same |

## Relied-upon claims

All ids below verified present in the imported 0.2.0 `provenance.yaml`
files (`status: confirmed`; all `source_category: black-box-observation`
except where noted). Normative unless marked.

| claim_id | used by |
|---|---|
| rmnode.signature-primary | FR-043-01 |
| rmnode.removes | FR-043-01 |
| rmnode.isolated | FR-043-01 |
| rmnode.class | FR-043-02 |
| rmnode.named | FR-043-03 |
| rmnode.weights | FR-043-04 |
| rmnode.empty-list | FR-043-05 |
| rmnode.all | FR-043-05 |
| rmnode.ignored-ids (**non-normative**) | FR-043-06 (documented choice) |
| addedge.signature-primary | FR-043-07 |
| addedge.edge-order | FR-043-07 |
| addedge.digraph-order | FR-043-07 |
| addedge.duplicates | FR-043-08 |
| addedge.self-loop | FR-043-08 |
| addedge.new-nodes | FR-043-09 |
| addedge.named | FR-043-09 |
| addedge.weights | FR-043-10 |
| addedge.scalar-expansion | FR-043-11 |
| addedge.error-weight-unweighted | FR-043-12 |
| addedge.error-length | FR-043-12 |
| rmedge.signature-primary | FR-043-13 |
| rmedge.by-endpoints | FR-043-13 |
| rmedge.multigraph | FR-043-13 |
| rmedge.digraph-direction | FR-043-13 |
| rmedge.absent | FR-043-13 |
| rmedge.by-index | FR-043-14 |
| rmedge.keeps-nodes | FR-043-15 |
| rmedge.weights | FR-043-15 |
| rmedge.error-index-range | FR-043-16 |
| rmedge.error-node-range | FR-043-16 |
| laplacian.signature-primary | FR-043-17 |
| laplacian.output-class | FR-043-17 |
| laplacian.values | FR-043-17 |
| laplacian.weights-ignored | FR-043-17 |
| laplacian.empty | FR-043-17 |
| laplacian.error-self-loop | FR-043-18 |
| laplacian.error-multigraph | FR-043-18 |
| laplacian.error-digraph | FR-043-18 |
| laplacian.numeric-input (**non-normative**) | U-043-1 (documented choice) |

Error identifiers: the exports record identifiers in the MATLAB namespace.
Per the maintainer's rule RunMat raises the final component in its own
per-function namespace (same mapping as feature 036):

| observed (export) | raised by RunMat |
|---|---|
| `…:addedge:NoWeights` | `RunMat:addedge:NoWeights` |
| `…:graphbuiltin:EqualNumel` (from addedge) | `RunMat:addedge:EqualNumel` |
| `…:findedge:EdgeBounds` (from rmedge) | `RunMat:rmedge:EdgeBounds` |
| `…:graph:InvalidNodeID` (from rmedge) | `RunMat:rmedge:InvalidNodeID` |
| `…:laplacian:SelfLoops` | `RunMat:laplacian:SelfLoops` |
| `…:laplacian:Multigraph` | `RunMat:laplacian:Multigraph` |
| `…:digraph:OnlyGraphSupported` (from laplacian) | `RunMat:laplacian:OnlyGraphSupported` |
| `UndefinedFunction` (laplacian, non-normative) | `RunMat:laplacian:UndefinedFunction` |

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| rmnode.q-ignored-ids | documented choice FR-043-06: ignored (settled by observed cases `out-of-range`, `unknown-name`) |
| rmnode.q-warning-coverage | implementation emits no warnings |
| rmnode.q-graph-variables | not applicable: RunMat graphs carry no extra variables |
| addedge.q-warning-coverage | implementation emits no warnings |
| addedge.q-graph-variables | not applicable (as above) |
| rmedge.q-warning-coverage | implementation emits no warnings |
| rmedge.q-graph-variables | not applicable (as above) |
| laplacian.q-warning-coverage | implementation emits no warnings |

Local documented choices U-043-1 … U-043-9 and deviations D-043-1/2 are in
`spec.md`.

## Source categories and independence

- Only the claims above are relied upon; 37 are normative, 2 are
  non-normative and back documented choices only.
- The edit operations are direct list manipulations of upstream's
  `GraphData`; the Laplacian is the textbook definition `L = D − A`. Nothing
  is derived from MATLAB internals.
- The stable tie order in `addedge` (FR-043-07) and every U-043-n item are
  inferences or choices, tagged `unresolved_choice_*` in tests and not
  asserted as MATLAB behaviour.
- No external implementation code was used (no licence to review).
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review MAY
  still be required.
