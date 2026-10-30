# Provenance Record — 036-graph-structure

| Field | `incidence` | `isisomorphic` |
|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same |
| Approved specification | `incidence` **0.3.0**, `status: approved` | `isisomorphic` **0.3.0**, `status: approved` |
| Specification revision (import pin) | **`8614c5b396851cdac1d1eb58f1cf5f7fa7f8909e`** (IMPORT-MANIFEST Batch 8, Tier B widening W2) | same |
| Review status | approved 2026-10-29 (source `docs/review/2026-10-29-tier-b-w2-widening-human-loop.md`, "Approve all 7") | same |
| Import date | 2026-10-29 | 2026-10-29 |
| Freshness check | 2026-10-29: in the source repo, after `git fetch --all`, `HEAD` = `origin/main` = `8614c5b`; `git diff --name-only 8614c5b HEAD` and `… origin/main` over both spec dirs empty; `git status --short` for both dirs clean | same |
| Applicable release | as recorded in the import (see the imported `provenance.yaml`) | same |
| Superseded | 0.2.0 (pin `104fb5f`, approval at `29d48cd`, IMPORT-MANIFEST Batch 6) | same |

## Relied-upon claims

All ids below verified present (`status: confirmed`, `normative: true`) in
the imported 0.3.0 `provenance.yaml` files.

| claim_id | category | used by |
|---|---|---|
| incidence.signature-primary | public-interface-fact | FR-036-01 |
| incidence.output-class | black-box-observation | FR-036-01 |
| incidence.shape | black-box-observation | FR-036-01 |
| incidence.signs | black-box-observation | FR-036-02 |
| incidence.no-warning | black-box-observation | FR-036-02 |
| incidence.edge-order | black-box-observation | FR-036-02 |
| incidence.multigraph | black-box-observation | FR-036-02 |
| incidence.weights-ignored | black-box-observation | FR-036-02 |
| incidence.error-self-loop-graph | black-box-observation | FR-036-03 |
| incidence.error-matrix | black-box-observation | FR-036-04 |
| isisomorphic.signature-primary | public-interface-fact | FR-036-05 |
| isisomorphic.output-class | black-box-observation | FR-036-05 |
| isisomorphic.true-cases | black-box-observation | FR-036-05 |
| isisomorphic.false-cases | black-box-observation | FR-036-05 |
| isisomorphic.no-warning | black-box-observation | FR-036-05 |
| isisomorphic.direction | black-box-observation | FR-036-05 |
| isisomorphic.weights-ignored | black-box-observation | FR-036-05 |
| isisomorphic.error-mixed | black-box-observation | FR-036-06 |
| isisomorphic.error-matrix | black-box-observation | FR-036-07 |

Error identifiers: the exports record identifiers in the MATLAB namespace.
Per the maintainer's rule, RunMat raises the same final component in its own
namespace: `RunMat:incidence:SelfLoops`, `RunMat:incidence:UndefinedFunction`,
`RunMat:isisomorphic:ClassMismatch`, `RunMat:isisomorphic:UndefinedFunction`.

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| incidence.q-warning-coverage | implementation emits no warnings |
| incidence.q-named-nodes | documented choice FR-036-10 (accepted; rows in node order) |
| isisomorphic.q-warning-coverage | implementation emits no warnings |
| isisomorphic.q-options | `RunMat:isisomorphic:UnsupportedOption` |
| isisomorphic.q-second-output | `RunMat:isisomorphic:UnsupportedOption` |

Local items U-036-1 and U-036-6 and deviations D-036-1/2 are in `spec.md`.

## Source categories and independence

- Only the 19 claims above are normative.
- The isomorphism search (FR-036-11) is a textbook backtracking algorithm
  with invariant pruning, derived from the mathematical definition of
  (multi)graph isomorphism. It is not derived from MATLAB internals.
- Named-node acceptance, extra-argument and arity handling are documented
  choices based on RunMat's graph representation; they are tagged in tests
  (`unresolved_choice_*`) and are not asserted as MATLAB-conformant.
- No external implementation code was used (no licence to review).
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review MAY
  still be required.
