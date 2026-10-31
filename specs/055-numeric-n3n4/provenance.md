# Provenance Record — 055-numeric-n3n4

| Field | `sec` | `orth` | `condest` |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `sec` **0.2.0**, `status: approved` | `orth` **0.2.0**, `status: approved` | `condest` **0.2.0**, `status: approved` |
| Specification revision (import pin) | **`1156c975db1f393b58252a44e04333a130211398`** (IMPORT-MANIFEST Batch 11, N3) | same as `sec` | **`fc9f80485f8b5df44a2a6b1ba6aec0e8274aa6c7`** (IMPORT-MANIFEST Batch 13, N4) |
| Review status | approved 2026-10-30 (source `docs/review/2026-10-30-n3-new-specs-human-loop.md`) | same as `sec` | approved 2026-10-30 (source `docs/review/2026-10-30-n4-new-specs-human-loop.md`) |
| Observations | 15 cases | 15 cases | 12 cases |
| Applicable release | R2026a (GLNXA64), as recorded in each imported `provenance.yaml` | same | same |

| Field | Value (all three) |
|---|---|
| Import date | 2026-10-30 |
| Selection | maintainer, 2026-10-30, `specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md` (F55; G1–G3 pre-approved) |
| Freshness check (start) | 2026-10-30: in `../matlab-interface-spec`, after `git fetch -q origin`, HEAD = origin/main = `c3305a5` (a tools-only commit after `fc9f804`). `git log 1156c97..origin/main` over `specifications/{sec,orth}` lists nothing, and `git log fc9f804..origin/main` over `specifications/condest` lists nothing; `git status --short` for those spec and experiment paths is clean (no drafts). Each imported file is byte-identical to `git show <pin>:specifications/<fn>/<file>`. |
| Freshness check (finish) | recorded in `validation.md` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/{sec,orth,condest}/{interface.yaml,behaviour.md,provenance.yaml,observations/*_observations.json}` |

## Relied-upon claims

All normative claims are `source_category: black-box-observation`,
`normative: true`, `status: confirmed`. Every id below exists in the
imported `provenance.yaml` (checked with `grep claim_id`; 9 + 10 + 9 = 28
ids, all listed).

| claim_id | category | used by |
|---|---|---|
| sec.signature-primary | observation, normative | FR-055-01 |
| sec.values | observation, normative | FR-055-01 |
| sec.pole | observation, normative | FR-055-02 |
| sec.shape | observation, normative | FR-055-03 |
| sec.nonfinite | observation, normative | FR-055-04 |
| sec.complex | observation, normative | FR-055-05 |
| sec.single | observation, normative | FR-055-06 |
| sec.error-logical | observation, normative | FR-055-07 |
| sec.error-none | observation, normative | FR-055-08 |
| orth.signature-primary | observation, normative | FR-055-09, FR-055-11 |
| orth.size-rank | observation, normative | FR-055-09 |
| orth.orthonormal | observation, normative | FR-055-10 |
| orth.class | observation, normative | FR-055-11, FR-055-12 |
| orth.empty | observation, normative | FR-055-13 |
| orth.second-arg | observation, normative | FR-055-14 |
| orth.diag | observation, **non-normative** | FR-055-17 |
| orth.basis-not-unique | **inference, non-normative, unresolved** | FR-055-17 |
| orth.error-sparse | observation, normative | FR-055-15 |
| orth.error-int | observation, normative | FR-055-16 |
| condest.signature-primary | observation, normative | FR-055-18 |
| condest.values | observation, normative | FR-055-19 |
| condest.vs-cond | observation, normative | FR-055-20 |
| condest.singular | observation, normative | FR-055-21 |
| condest.empty | observation, normative | FR-055-22 |
| condest.two-out | observation, normative | FR-055-18, FR-055-23 |
| condest.estimate | **inference, non-normative, unresolved** | D-055-3 |
| condest.error-nonsquare | observation, normative | FR-055-24 |
| condest.error-char | observation, normative | FR-055-25 |

## Unresolved questions carried

`sec.q-warning-coverage`, `orth.q-warning-coverage`, `orth.q-default-tol`,
`orth.q-second-arg`, `orth.q-error-ids`, `condest.q-warning-coverage`,
`condest.q-error-ids`, `condest.q-larger`, `condest.q-exactness`.
`spec.md` ("Unresolved behaviour") records how each is handled.

## Source categories and independence

- FR-055-01..16 and 18..25 come from black-box observations. The observed
  identifiers are in the `MATLAB:` namespace, and RunMat raises them in its
  own namespace (D-055-1).
- FR-055-17 records non-normative observations and an inference; they are
  not requirements.
- FR-055-26..33 are **independently derived documented choices** or
  explicit `UnsupportedOption` dispositions. Tests tag them
  `documented_choice_*` / `unresolved_*`.
- Algorithms, written for this feature from textbook sources:
  - sec: the identity `sec x = 1/cos x` (real `f64::cos`, complex
    `num_complex::Complex64::cos`).
  - orth: leading left singular vectors of a thin SVD above a rank
    threshold (standard numerical-rank construction; Golub & Van Loan,
    *Matrix Computations*). The SVD is nalgebra's (Apache-2.0, already a
    RunMat dependency).
  - condest: `norm(A,1) * norm(inv(A),1)` with an LU inverse (nalgebra);
    `v` from the column of `inv(A)` of largest 1-norm (Higham, *Accuracy and
    Stability of Numerical Algorithms*, 1-norm condition identities). This
    is not an implementation of any estimator algorithm.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
