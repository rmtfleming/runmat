# Provenance Record — 063-rat

| Field | `rat` |
|---|---|
| Source repository | `matlab-interface-spec` (local) |
| Approved specification | `rat` **0.2.0**, `status: approved` |
| Specification revision (import pin) | **`1c91aa1`** (IMPORT-MANIFEST Batch 14, N5) |
| Review status | approved 2026-10-30 (source `docs/review/2026-10-30-n5-new-specs-human-loop.md`) |
| Observations | 19 cases |
| Import date | 2026-10-31 (imported copy under `specs/imports/matlab-interface-spec/specifications/rat/`) |
| Selection | maintainer, 2026-10-31, `specs/001-clean-room-matlab-builtins/selection-proposal-n5.md` (F63; G1–G3 pre-approved; `rat.q-complex`: "complex input supported as observed") |
| Applicable release | R2026a (as recorded in the imported `provenance.yaml`) |

## Freshness check (start)

2026-10-31, in `../matlab-interface-spec` after `git fetch -q origin`
(rc 0): `HEAD` = `origin/main` = `1c91aa1` (the pin).
`git log 1c91aa1..origin/main -- specifications/rat` is empty;
`git status --short` shows no change under `specifications/rat` or
`experiments/rat` (tracked at the pin); `diff -r` of the imported copy
against the source tree reports no difference. Final check:
`validation.md`.

## Relied-upon claims

All ids below verified present in the imported 0.2.0 `provenance.yaml`
(`source_category: black-box-observation`, `status: confirmed`).

| claim_id | normative | used by |
|---|---|---|
| rat.signature-primary | yes | FR-063-01 |
| rat.simple | yes | FR-063-01 |
| rat.default-tol | yes | FR-063-02 |
| rat.tol | yes | FR-063-03 |
| rat.elementwise | yes | FR-063-04 |
| rat.class | yes | FR-063-05 |
| rat.text | yes | FR-063-07 |
| rat.nonfinite | yes | FR-063-06 |
| rat.complex | **no** | D-063-3, U-063-4 (supported as observed, maintainer disposition) |
| rat.error-char | yes | FR-063-08 |

Error identifiers (D-063-1):

| observed (export) | raised by RunMat |
|---|---|
| `MATLAB:norm:inputType` | `RunMat:norm:inputType` |

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| rat.q-warning-coverage | implementation emits no warnings |
| rat.q-error-ids | raised as observed under `RunMat:` (D-063-1) |
| rat.q-default-tol | documented choice U-063-2 |
| rat.q-complex | supported as observed (D-063-3), layout U-063-4 |

## Source categories and independence

- Only the claims above are relied upon (all 10 claims of the export; 9
  normative, 1 non-normative backing a supported-as-observed form).
- Algorithm: the textbook continued-fraction expansion with nearest-integer
  partial quotients and the standard convergent recurrence. The default
  tolerance `1e-6*|x|` is RunMat's own choice, validated only against the
  observed results; it is not a claim about how MATLAB chooses its
  tolerance.
- No external implementation code was used (no licence to review).
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review MAY
  still be required.
