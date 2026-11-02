# Provenance Record — 067-numeric-n6

| Field | `normest` | `lsqnonneg` |
|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same |
| Approved specification | `normest` **0.2.0**, `status: approved` | `lsqnonneg` **0.2.0**, `status: approved` |
| Specification revision (import pin) | **`e3642060f88131d84cb6280bc986a5cdee52d28f`** (IMPORT-MANIFEST Batch 15, N6) | same |
| Review status | approved 2026-11-02 (source `docs/review/2026-11-02-n6-new-specs-human-loop.md`) | same |
| Observations | 20 cases | 18 cases |
| Applicable release | R2026a (GLNXA64), as recorded in each imported `provenance.yaml` | same |

| Field | Value (both) |
|---|---|
| Import date | 2026-11-02 (IMPORT-MANIFEST Batch 15) |
| Selection | maintainer, `specs/001-clean-room-matlab-builtins/selection-proposal-n6.md` (F67; G1–G3 pre-approved; dispositions for `normest.q-exactness`, `normest.q-default-tol`, `lsqnonneg` `output.algorithm`/`message`, `lsqnonneg.q-nonunique` and `q-error-ids` recorded there) |
| Freshness check (start) | 2026-11-02: in `../matlab-interface-spec`, after `git fetch origin`, HEAD = origin/main = `20825f8`. `git log e364206..origin/main` lists one commit (`20825f8`, `docs/prioritization/cobra-matlab-builtins.csv` only); nothing under `specifications/{normest,lsqnonneg}` changed. `git status --short` shows no drafts for those paths. Each imported file is byte-identical to `git show e364206:specifications/<fn>/<file>` (`cmp`, 8 files). |
| Freshness check (finish) | recorded in `validation.md` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/{normest,lsqnonneg}/{interface.yaml,behaviour.md,provenance.yaml,observations/*_observations.json}` |

## Relied-upon claims

Every id below exists in the imported `provenance.yaml` (checked with
`grep claim_id`; 12 + 16 = 28 ids, all listed). All are
`source_category: black-box-observation`, `status: confirmed`; normativity
as marked.

| claim_id | normative | used by |
|---|---|---|
| normest.signature-primary | yes | FR-067-01 |
| normest.output-class | yes | FR-067-02 |
| normest.default-accuracy | yes | FR-067-03 |
| normest.cobra-form | yes | FR-067-04 |
| normest.tol | yes | FR-067-05 |
| normest.zero | yes | FR-067-06 |
| normest.scalar-nan | yes | FR-067-07 |
| normest.char | **no** | FR-067-11, D-067-3 |
| normest.iteration-count | **no** | FR-067-12 |
| normest.error-int | yes | FR-067-08 |
| normest.error-3d | yes | FR-067-09 |
| normest.error-no-arg | yes | FR-067-10 |
| lsqnonneg.signature-primary | yes | FR-067-20 |
| lsqnonneg.output-class | yes | FR-067-21 |
| lsqnonneg.cobra-form | yes | FR-067-22 |
| lsqnonneg.solutions | yes | FR-067-23 |
| lsqnonneg.residual | yes | FR-067-24 |
| lsqnonneg.lambda | yes | FR-067-25 |
| lsqnonneg.output-struct | yes | FR-067-26 |
| lsqnonneg.options | yes | FR-067-27 |
| lsqnonneg.empty | yes | FR-067-28 |
| lsqnonneg.nonunique | **no** | FR-067-34, D-067-4 |
| lsqnonneg.iterations | **no** | FR-067-35 |
| lsqnonneg.error-single | yes | FR-067-29 |
| lsqnonneg.error-complex | yes | FR-067-30 |
| lsqnonneg.error-size | yes | FR-067-31 |
| lsqnonneg.error-row-d | yes | FR-067-32 |
| lsqnonneg.error-one-arg | yes | FR-067-33 |

## Unresolved questions carried

`normest.q-warning-coverage`, `normest.q-exactness`,
`normest.q-default-tol`, `normest.q-char`, `normest.q-error-ids`,
`lsqnonneg.q-warning-coverage`, `lsqnonneg.q-exactness`,
`lsqnonneg.q-output-struct`, `lsqnonneg.q-optimality`,
`lsqnonneg.q-nonunique`, `lsqnonneg.q-options`, `lsqnonneg.q-error-ids`.
`spec.md` ("Unresolved behaviour") records how each is handled.

## Source categories and independence

- FR-067-01..10 and 20..33 come from black-box observations. The observed
  identifiers are in the `MATLAB:` namespace, and RunMat raises them in its
  own namespace (D-067-1).
- FR-067-11, 12, 34, 35 record the four non-normative observations; they
  are not requirements.
- FR-067-13..16 and 36..41 are **independently derived documented
  choices** or `UnsupportedOption` dispositions; tests tag them
  `documented_choice_*` / `unresolved_*`.
- Algorithms, from textbook sources:
  - normest: the power method for the dominant eigenvalue of `S'*S`
    (Golub & Van Loan, *Matrix Computations*, 4th ed., §7.3.1 and §8.6),
    with RunMat's own start vector, stopping rule and default tolerance.
  - lsqnonneg: algorithm NNLS of Lawson & Hanson, *Solving Least Squares
    Problems*, Prentice-Hall 1974 (SIAM reprint 1995), ch. 23; subproblems by
    nalgebra's SVD (Apache-2.0, already a RunMat dependency).
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
