# Provenance Record — 061-sparse-linalg-n5

| Field | `lsqminnorm` | `sprandn` | `sprank` |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `lsqminnorm` **0.2.0**, `status: approved` | `sprandn` **0.2.0**, `status: approved` | `sprank` **0.2.0**, `status: approved` |
| Specification revision (import pin) | **`1c91aa1c9bda12a34792d49dff22727d9fcbcc20`** (IMPORT-MANIFEST Batch 14, N5) | same | same |
| Review status | approved 2026-10-30 (source `docs/review/2026-10-30-n5-new-specs-human-loop.md`) | same | same |
| Observations | 17 cases | 11 cases | 14 cases |
| Applicable release | R2026a (GLNXA64), as recorded in each imported `provenance.yaml` | same | same |

| Field | Value (all three) |
|---|---|
| Import date | 2026-10-31 (IMPORT-MANIFEST Batch 14) |
| Selection | maintainer, `specs/001-clean-room-matlab-builtins/selection-proposal-n5.md` (F61; G1–G3 pre-approved; dispositions for `lsqminnorm.q-tol-rule` and `sprandn.q-values` recorded there) |
| Freshness check (start) | 2026-10-31: in `../matlab-interface-spec`, after `git fetch origin`, HEAD = origin/main = `1c91aa1` (the pin). `git log 1c91aa1..origin/main` is empty; `git status --short` shows nothing under `specifications/{lsqminnorm,sprandn,sprank}` (no drafts). Each imported file is byte-identical to `git show 1c91aa1:specifications/<fn>/<file>` (`cmp`, 12 files). |
| Freshness check (finish) | recorded in `validation.md` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/{lsqminnorm,sprandn,sprank}/{interface.yaml,behaviour.md,provenance.yaml,observations/*_observations.json}` |

## Relied-upon claims

Every id below exists in the imported `provenance.yaml` (checked with
`grep claim_id`; 12 + 10 + 8 = 30 ids, all listed). Unless marked, the
claim is `source_category: black-box-observation`, `normative: true`,
`status: confirmed`.

| claim_id | category | used by |
|---|---|---|
| lsqminnorm.signature-primary | observation, normative | FR-061-01 |
| lsqminnorm.vs-pinv | observation, normative | FR-061-01, FR-061-07 |
| lsqminnorm.full-output | observation, normative | FR-061-02 |
| lsqminnorm.cobra-form | observation, normative | FR-061-03 |
| lsqminnorm.zero | observation, normative | FR-061-04 |
| lsqminnorm.single | observation, normative | FR-061-05 |
| lsqminnorm.empty | observation, normative | FR-061-06 |
| lsqminnorm.tol | observation, normative | FR-061-07 |
| lsqminnorm.tol-rule | **inference, non-normative, unresolved** | FR-061-08, D-061-3 |
| lsqminnorm.error-rows | observation, normative | FR-061-09 |
| lsqminnorm.error-char | observation, normative | FR-061-10 |
| lsqminnorm.error-one-arg | observation, normative | FR-061-11 |
| sprandn.signature-primary | observation, normative | FR-061-29 |
| sprandn.structure | observation, normative | FR-061-29 |
| sprandn.nnz-bound | observation, normative | FR-061-30 |
| sprandn.zero-density | observation, normative | FR-061-31 |
| sprandn.pattern | observation, normative | FR-061-32 |
| sprandn.rc | observation, normative | FR-061-33 |
| sprandn.density-out-of-range | observation, **non-normative** | FR-061-36 |
| sprandn.values | **inference, non-normative, unresolved** | FR-061-37, D-061-5 |
| sprandn.error-noninteger | observation, normative | FR-061-34 |
| sprandn.error-two-args | observation, normative | FR-061-35 |
| sprank.signature-primary | observation, normative | FR-061-18 |
| sprank.structural | observation, normative | FR-061-19 |
| sprank.matching | observation, normative | FR-061-20 |
| sprank.zero-empty | observation, normative | FR-061-21 |
| sprank.inputs | observation, normative | FR-061-22 |
| sprank.class | observation, normative | FR-061-23 |
| sprank.error-char | observation, normative | FR-061-24 |
| sprank.error-no-arg | observation, normative | FR-061-25 |

## Unresolved questions carried

`lsqminnorm.q-warning-coverage`, `lsqminnorm.q-tol-rule`,
`lsqminnorm.q-error-ids`, `lsqminnorm.q-exactness`,
`sprandn.q-warning-coverage`, `sprandn.q-error-ids`, `sprandn.q-values`,
`sprandn.q-density`, `sprank.q-warning-coverage`, `sprank.q-error-ids`.
`spec.md` ("Unresolved behaviour") records how each is handled.

## Source categories and independence

- FR-061-01..07, 09..11, 18..25 and 29..35 come from black-box
  observations. The observed identifiers are in the `MATLAB:` namespace,
  and RunMat raises them in its own namespace (D-061-1).
- FR-061-08, FR-061-36 and FR-061-37 record non-normative observations and
  inferences; they are not requirements.
- FR-061-12..17, 26..28 and 38..40 are **independently derived documented
  choices** or explicit `UnsupportedOption` dispositions. Tests tag them
  `documented_choice_*` / `unresolved_*`.
- Algorithms, written for this feature from textbook sources:
  - lsqminnorm: the minimum-norm least-squares solution from a thin SVD
    with a numerical-rank threshold, and a Householder QR factorisation
    with column pivoting truncated at a rank threshold followed by the
    minimum-norm solution of the truncated system (Golub & Van Loan,
    *Matrix Computations*, ch. 5). The SVD is nalgebra's (Apache-2.0,
    already a RunMat dependency); the pivoted QR is written here.
  - sprank: size of a maximum bipartite matching, Hopcroft & Karp, "An
    n^{5/2} algorithm for maximum matchings in bipartite graphs", SIAM J.
    Comput. 2(4), 1973.
  - sprandn: uniform sampling without replacement and normal draws from
    RunMat's existing random stream; the `rc` construction is RunMat's
    existing `sprand` code (diagonal of singular values plus random plane
    rotations).
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
