# Provenance Record — 023-spalloc

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `spalloc` 0.2.0 — `status: approved` |
| Specification revision | Commit pin: `4445294` of `matlab-interface-spec` ("Approve Tier B batch 2 after human-loop review"; Tier B batch 2; see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, batch-4 entry) |
| Review status | Approved 2026-07-22 by repository maintainer (source `docs/review/2026-07-22-tier-b-batch2-human-loop.md`, "all 12 approved"; `spalloc` among the four spec-side corrections before promotion) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |

Imported artefacts (byte-identical to source at the pinned commit, per the
import manifest):
`specs/imports/matlab-interface-spec/specifications/spalloc/{interface.yaml,
behaviour.md, provenance.yaml, observations/spalloc_observations.json}`.

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| spalloc.signature-primary | public-interface-fact | yes |
| spalloc.output-class | black-box-observation | yes |
| spalloc.all-zero-sparse | black-box-observation | yes |

Backing observations (1 case, `errored: false`, class `double`):
`basic` — `spalloc(3, 3, 4)` → `sparse(zeros(0,1), zeros(0,1), zeros(0,1), 3, 3)`
(a 3×3 sparse `double`, `issparse` true, `isreal` true, no stored nonzeros),
size `[3, 3]`.

## Independently derived inferences (not observations)

The approved export records **no** unresolved questions and only the single
`basic` case. The following are RunMat independent choices, tagged
`unresolved_choice_*` in tests; they are not presented as observed MATLAB
behaviour:

- `nz` is a capacity hint only. RunMat's compressed-sparse-column
  representation has no separate reserved capacity, so the hint cannot change
  the observable result (nnz stays 0); it is accepted and validated but
  otherwise ignored.
- Dimensions and `nz` are accepted from the same scalar-like value kinds as the
  other sparse constructors (`Num`, `Int`, `Bool`, 1-element tensor) via the
  shared `scalar_f64` / `size_from_f64` helpers.
- Non-integer, negative, non-finite, and non-scalar dimensions, invalid `nz`,
  and argument counts other than three are rejected with
  `RunMat:spalloc:InvalidArgument`.

No external algorithm source is required (Principle X): the result is the
empty CSC matrix constructor `SparseTensor::zeros(m, n)`. No MathWorks source,
documentation prose, or examples were consulted. Independent legal review MAY
still be required (constitution Principle X).

Unresolved questions carried (RunMat-side): `spalloc.q-nz-capacity`,
`spalloc.q-dims`, `spalloc.q-invalid-dims`, `spalloc.q-arity`.
