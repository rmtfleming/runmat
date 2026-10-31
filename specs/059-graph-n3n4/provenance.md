# Provenance Record — 059-graph-n3n4

| Field | `isomorphism` | `allpaths` | `inedges` |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `isomorphism` **0.2.0**, `status: approved` | `allpaths` **0.2.0**, approved | `inedges` **0.2.0**, approved |
| Specification revision (import pin) | **`1156c975db1f393b58252a44e04333a130211398`** (IMPORT-MANIFEST Batch 11, N3) | **`fc9f80485f8b5df44a2a6b1ba6aec0e8274aa6c7`** (IMPORT-MANIFEST Batch 13, N4) | same as `allpaths` |
| Review status | approved 2026-10-30 (source `docs/review/2026-10-30-n3-new-specs-human-loop.md`) | approved 2026-10-30 (source `docs/review/2026-10-30-n4-new-specs-human-loop.md`) | same as `allpaths` |
| Observations | 15 cases | 11 cases | 9 cases |
| Import date | 2026-10-30 (imported copies under `specs/imports/matlab-interface-spec/specifications/`) | same | same |
| Selection | maintainer, 2026-10-30, `specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md` (F59; G1–G3 pre-approved) | same | same |
| Applicable release | as recorded in each imported `provenance.yaml` (R2026a) | same | same |

## Freshness check (start)

2026-10-30, in `../matlab-interface-spec` after `git fetch -q origin`:
`HEAD` = `origin/main` = `c3305a5` (later than both pins).
`git log 1156c97..origin/main -- specifications/isomorphism` and
`git log fc9f804..origin/main -- specifications/{allpaths,inedges}` are
empty (the last commits touching the three directories are the pins
themselves). `git status --short` shows no change under the three
`specifications/` directories (untracked `experiments/` folders exist for
other functions only; none for these three). The imported copies are
byte-identical to the source working tree (`sha256sum` comparison).
Final check: `validation.md`.

## Relied-upon claims

All ids below verified present in the imported 0.2.0 `provenance.yaml`
files (`source_category: black-box-observation`, `status: confirmed`,
`normative: true` unless marked).

| claim_id | used by |
|---|---|
| isomorphism.signature-primary | FR-059-01 |
| isomorphism.output-class | FR-059-01 |
| isomorphism.orientation | FR-059-02 |
| isomorphism.not-isomorphic | FR-059-03 |
| isomorphism.empty-graphs | FR-059-03 |
| isomorphism.edgeperm | FR-059-04 |
| isomorphism.node-variables | FR-059-05 |
| isomorphism.edge-variables | FR-059-05 |
| isomorphism.error-mixed | FR-059-06 |
| isomorphism.error-unknown-nodevar | FR-059-06 |
| isomorphism.non-graph (**non-normative**) | U-059-1 (documented choice) |
| isomorphism.non-unique (**non-normative, unresolved, inference**) | U-059-4(a) (documented choice) |
| allpaths.signature-primary | FR-059-07 |
| allpaths.paths | FR-059-07 |
| allpaths.digraph | FR-059-08 |
| allpaths.unreachable | FR-059-08 |
| allpaths.same-node | FR-059-09 |
| allpaths.names | FR-059-10 |
| allpaths.edgepaths | FR-059-11 |
| allpaths.error-node-range | FR-059-12 |
| allpaths.error-unknown-name | FR-059-12 |
| allpaths.non-graph (**non-normative**) | U-059-1 (documented choice) |
| allpaths.order (**non-normative**) | U-059-6 (documented choice) |
| inedges.signature-primary | FR-059-13 |
| inedges.outputs | FR-059-13 |
| inedges.order | FR-059-13 |
| inedges.none | FR-059-14 |
| inedges.selfloop | FR-059-14 |
| inedges.names | FR-059-15 |
| inedges.error-undirected | FR-059-16 |
| inedges.error-node-range | FR-059-16 |
| inedges.error-two-nodes | FR-059-16 |

Error identifiers: the exports record identifiers in the MATLAB namespace.
RunMat raises the final component in its own per-function namespace (same
mapping as features 036 and 043):

| observed (export) | raised by RunMat |
|---|---|
| `…:isomorphism:ClassMismatch` | `RunMat:isomorphism:ClassMismatch` |
| `…:isomorphism:NodeVarNotInFirstGraph` | `RunMat:isomorphism:NodeVarNotInFirstGraph` |
| `…:graph:InvalidNodeID` (from allpaths) | `RunMat:allpaths:InvalidNodeID` |
| `…:graph:UnknownNodeName` (from allpaths) | `RunMat:allpaths:UnknownNodeName` |
| `…:graph:NoInEdgesUndir` (from inedges) | `RunMat:inedges:NoInEdgesUndir` |
| `…:digraph:InvalidNodeID` (from inedges) | `RunMat:inedges:InvalidNodeID` |
| `…:graphbuiltin:InvalidNodeScalar` (from inedges) | `RunMat:inedges:InvalidNodeScalar` |
| `UndefinedFunction` (isomorphism, allpaths; dispatch level, non-normative) | `RunMat:<fn>:UndefinedFunction` (U-059-1) |

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| isomorphism.q-warning-coverage | implementation emits no warnings |
| isomorphism.q-more | U-059-4(c) multisets for parallel edges/self-loops; U-059-4(d) several variables allowed |
| allpaths.q-warning-coverage | implementation emits no warnings |
| allpaths.q-more | U-059-2 (more than three arguments → `UnsupportedOption`), U-059-6 (multigraphs, self-loops) |
| inedges.q-warning-coverage | implementation emits no warnings |

Local documented choices U-059-1 … U-059-7 and deviations D-059-1 … 3 are in
`spec.md`.

## Source categories and independence

- Only the claims above are relied upon (all 32 claims of the three
  exports); 28 are normative, 4 are
  non-normative and back documented choices only.
- Algorithms are textbook: backtracking graph-isomorphism search with
  degree/label invariants for `isomorphism`; depth-first enumeration of
  simple paths for `allpaths`; a filter over the stored edge list for
  `inedges`. `isomorphism` uses its own labelled matcher rather than
  feature 036's unlabelled `isisomorphic` search (which returns only a
  boolean and has no node/edge labels); `structure.rs` was not modified.
- No external implementation code was used (no licence to review).
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review MAY
  still be required.
