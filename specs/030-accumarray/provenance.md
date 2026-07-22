# Provenance Record — 030-accumarray

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `accumarray` 0.2.0 — `status: approved` (observed behaviour confirmed) |
| Specification version | 0.2.0 |
| Specification revision | Pinned to source commit **`1b019121bb747e63d69a3eb005534cb60669e5f4`** ("Clean up stale SKELETON labels"; Tier B batch 3 / IMPORT-MANIFEST.md Batch 5). Approved at source commit `67eaf83`; the only post-approval change to this spec is a cosmetic `notes`-field edit in the observation JSON (no case data, values, classes, or claims changed), still `status: approved` at the pinned HEAD. |
| Review status | Approved 2026-07-22 (source `docs/review/2026-07-22-tier-b-batch3-human-loop.md`; all 12 batch-3 specs approved) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |

## Relied-upon claims

| claim_id | category | normative | used by |
|---|---|---|---|
| accumarray.signature-primary | public-interface-fact | yes | FR-030-01 |
| accumarray.output-class | black-box-observation | yes | FR-030-01 |
| accumarray.sum-by-subscript | black-box-observation | yes | FR-030-01 |

Backing observation: `observations/accumarray_observations.json`, case `sum`
(`accumarray([1; 2; 1], [10; 20; 30])` → value `[40;20]`, class `double`, size
`[2,1]`, real, non-sparse), R2026a / GLNXA64.

## Unresolved questions carried (returned to specification side)

| id | question | disposition in this feature |
|---|---|---|
| accumarray.q-func | The custom accumulation-function form. | Rejected clearly (`RunMat:accumarray:CustomFunctionUnsupported`); only default sum accepted. |
| accumarray.q-size | The size and fill-value options. | `sz`/`fillval` implemented for the single-subscript `[m 1]` column form as documented independent choices; multi-column (N-D) and `issparse` rejected clearly. |

## Source categories and independence

- Only the three claims above are normative and observation/interface backed.
- `sz`, `fillval`, scalar-`vals` broadcast, positive-integer validation, empty
  handling, and the 1×1→scalar collapse are **independently derived inferences
  / documented choices**, marked as such in `spec.md` and in the test tiers
  (`unresolved_choice_*`, `rejects_*`). None is asserted as MATLAB-conformant.
- No MathWorks source, documentation prose, or examples were consulted; MATLAB
  was not invoked in this repository.
