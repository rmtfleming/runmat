# Provenance Record — 042-sparse-orderings

| Field | `amd` | `etree` | `symbfact` |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `amd` **0.2.0**, `status: approved` | `etree` **0.2.0**, `status: approved` | `symbfact` **0.2.0**, `status: approved` |
| Specification revision (import pin) | **`c65e2382fa2420416439f8342333af77dcaee5b3`** | same | same |
| Review status | approved 2026-10-29, batch N1 (source `docs/review/2026-10-29-n1-new-specs-human-loop.md` in `matlab-interface-spec`) | same | same |
| Import date | 2026-10-29 (imported into `specs/imports/`) | same | same |
| Imported files | `specs/imports/matlab-interface-spec/specifications/amd/{interface.yaml,behaviour.md,provenance.yaml,observations/amd_observations.json}` (8 cases) | `…/etree/…` (14 cases) | `…/symbfact/…` (9 cases) |
| Applicable release | as recorded in the imported `provenance.yaml` | same | same |

## Freshness check (2026-10-30, start of work)

In `../matlab-interface-spec` (read-only): `git fetch -q --all`;
`git diff --name-only c65e238 HEAD -- specifications/{amd,etree,symbfact,balance}`
and the same against `origin/main` were both empty; `git status --short` for
those directories was clean; HEAD = origin/main = `8f72edb`. The import is
current. The end-of-work recheck is recorded in `validation.md`.

## Relied-upon claims

All 25 ids below are verified present in the imported `provenance.yaml`
files. All are `source_category: black-box-observation`.

| claim_id | normative | cases | used by |
|---|---|---|---|
| amd.signature-primary | true | perm | FR-042-01 |
| amd.output-class | true | perm, class, diag | FR-042-01 |
| amd.permutation | true | perm, arrow, diag | FR-042-02 |
| amd.dense-input | true | arrow, dense-input | FR-042-03 |
| amd.trivial-sizes | true | empty, scalar | FR-042-04 |
| amd.specific-orderings | **false** | perm, arrow, diag | FR-042-15 (reference only, not asserted) |
| amd.error-nonsquare | true | err-nonsquare | FR-042-05 |
| etree.signature-primary | true | tridiagonal | FR-042-06 |
| etree.output-class | true | tridiagonal, diag, pentadiag | FR-042-06 |
| etree.parent-vector | true | tridiagonal, arrow, pentadiag, diag | FR-042-06 |
| etree.upper-triangle | true | upper-only, lower-only, tridiagonal | FR-042-07 |
| etree.sym-default | true | sym, tridiagonal | FR-042-06 |
| etree.dense-input | true | dense-input, tridiagonal | FR-042-07 |
| etree.col | true | col | FR-042-08 |
| etree.postorder | true | postorder, postorder-arrow, postorder-branching | FR-042-09 |
| etree.branching | true | postorder-branching | FR-042-06 |
| etree.empty | true | empty | FR-042-10 |
| etree.error-nonsquare | true | err-nonsquare | FR-042-10 |
| symbfact.signature-primary | true | tridiagonal | FR-042-11 |
| symbfact.output-class | true | tridiagonal, class | FR-042-11 |
| symbfact.counts | true | tridiagonal, arrow, arrow-reversed, diag, class, outputs | FR-042-11 |
| symbfact.outputs | true | outputs | FR-042-12 |
| symbfact.dense-input | true | dense-input, tridiagonal | FR-042-13 |
| symbfact.col | true | col | FR-042-14 |
| symbfact.error-nonsquare | true | err-nonsquare | FR-042-14 |

## Unresolved questions carried (returned to the specification side)

| id | disposition in this feature |
|---|---|
| amd.q-warning-coverage | no warnings emitted |
| amd.q-more-args | `RunMat:amd:UnsupportedOption` |
| amd.q-ordering | ordering non-normative (maintainer decision); minimum-degree choice documented |
| etree.q-warning-coverage | no warnings emitted |
| etree.q-postorder-general | documented choice: DFS postorder, roots and children in increasing index order |
| symbfact.q-warning-coverage | no warnings emitted |
| symbfact.q-more-options | `RunMat:symbfact:UnsupportedOption` for any option other than `'col'` and for extra arguments |

## Source categories and independence

- **Black-box observations**: the 25 normative claims above. The observed
  error identifiers `MATLAB:square`, `MATLAB:etree:InvalidInput` and
  `MATLAB:symbfact:InvalidInput` are emitted as `RunMat:square`,
  `RunMat:etree:InvalidInput` and `RunMat:symbfact:InvalidInput` (namespace
  decision, see `spec.md`).
- **Non-normative observation**: `amd.specific-orderings`. It is recorded
  only; no test asserts it.
- **Independently derived inferences / documented choices**: the ordering
  heuristic, the sibling order of the postorder, the definition of `h` beyond
  the one observed value, `symbfact` using the upper triangle, accepted input
  classes, the option matching rules, too-many-outputs errors. They are
  tagged `documented_choice_*` or `unresolved_*` in tests and are not claimed
  to be MATLAB-conformant.

## External algorithm sources (Constitution Principle X)

- Minimum-degree ordering: W. F. Tinney and J. W. Walker, "Direct solutions
  of sparse network equations by optimally ordered triangular
  factorization", Proc. IEEE 55(11), 1967; A. George and J. W. H. Liu,
  *Computer Solution of Large Sparse Positive Definite Systems*,
  Prentice-Hall, 1981. Implemented as an exact minimum-degree elimination on
  an explicit graph. It is not the approximate-minimum-degree algorithm of
  Amestoy, Davis and Duff, and no SuiteSparse source was consulted.
- Elimination tree (with path compression) and column elimination tree of
  `A'*A` without forming `A'*A`: J. W. H. Liu, "The role of elimination
  trees in sparse factorization", SIAM J. Matrix Anal. Appl. 11(1), 1990;
  T. A. Davis, *Direct Methods for Sparse Linear Systems*, SIAM, 2006
  (textbook algorithm descriptions).
- Symbolic factorization: the structure of column `j` of `L` is the
  structure of the strictly lower part of column `j` of `A`, joined with the
  structures of the children of `j` in the elimination tree minus the
  children themselves (George & Liu 1981; Liu 1990).
- All Rust code is original. No MathWorks source, documentation prose or
  examples were consulted, and MATLAB was not invoked from this repository.
