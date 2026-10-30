# Provenance Record — 047-balance

| Field | `balance` |
|---|---|
| Source repository | `matlab-interface-spec` (local) |
| Approved specification | `balance` **0.2.0**, `status: approved` |
| Specification revision (import pin) | **`c65e2382fa2420416439f8342333af77dcaee5b3`** |
| Review status | approved 2026-10-29, batch N1 (source `docs/review/2026-10-29-n1-new-specs-human-loop.md` in `matlab-interface-spec`) |
| Import date | 2026-10-29 (imported into `specs/imports/`) |
| Imported files | `specs/imports/matlab-interface-spec/specifications/balance/{interface.yaml,behaviour.md,provenance.yaml,observations/balance_observations.json}` (11 cases) |
| Applicable release | as recorded in the imported `provenance.yaml` |

## Freshness check (2026-10-30, start of work)

In `../matlab-interface-spec` (read-only): `git fetch -q --all`;
`git diff --name-only c65e238 HEAD -- specifications/{amd,etree,symbfact,balance}`
and the same against `origin/main` were both empty; `git status --short` for
those directories was clean; HEAD = origin/main = `8f72edb`. The end-of-work
recheck is recorded in `validation.md`.

## Relied-upon claims

All 10 ids are verified present in the imported `provenance.yaml`. All are
`source_category: black-box-observation`.

| claim_id | normative | cases | used by |
|---|---|---|---|
| balance.signature-primary | true | single-output | FR-047-01 |
| balance.output-class | true | single-output, single-class | FR-047-01 |
| balance.similarity | true | two-outputs, permutation, eig-preserved | FR-047-02 |
| balance.power-of-two | true | two-outputs | FR-047-03 |
| balance.three-outputs | true | three-outputs, two-outputs | FR-047-04 |
| balance.already-balanced | true | balanced, scalar | FR-047-05 |
| balance.noperm | true | noperm, single-output | FR-047-06 |
| balance.observed-values | **false** | single-output, two-outputs, permutation | FR-047-08 (reference only, not asserted) |
| balance.error-nonsquare | true | err-nonsquare | FR-047-07 |
| balance.error-sparse | true | err-sparse | FR-047-07 |

## Unresolved questions carried (returned to the specification side)

| id | disposition in this feature |
|---|---|
| balance.q-warning-coverage | no warnings emitted |
| balance.q-exact-values | not asserted (maintainer decision); algorithm documented |
| balance.q-noperm-permuting | documented choice: `'noperm'` scales only, `P = (1:n)'` |
| balance.q-complex | `RunMat:balance:UnsupportedOption` |

## Source categories and independence

- **Black-box observations**: the 9 normative claims above. Observed
  identifiers `MATLAB:square` and `MATLAB:balance:noBalanceSparse` are
  emitted as `RunMat:square` and `RunMat:balance:noBalanceSparse`.
- **Non-normative observation**: `balance.observed-values`. It is recorded
  only; no test asserts it.
- **Independently derived inferences / documented choices**: the algorithm,
  the `T = I(:,P)*diag(S)` relation for permuted matrices, `'noperm'` on a
  permutable matrix, input-class rules, empty and non-finite handling, the
  too-many-outputs error. They are tagged `documented_choice_*` or
  `unresolved_*` in tests.

## External algorithm source (Constitution Principle X)

- B. N. Parlett and C. Reinsch, "Balancing a matrix for calculation of
  eigenvalues and eigenvectors", Numerische Mathematik 13, 293–304, 1969
  (permutation to isolate eigenvalues, then power-of-radix diagonal scaling
  with the 0.95 convergence test). The Rust implementation is original and
  written from the published description. No MathWorks source,
  documentation prose or examples were consulted, and MATLAB was not
  invoked from this repository.
