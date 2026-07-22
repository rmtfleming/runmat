# Provenance Record — 026-append

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `append` 0.2.0 — `status: approved` |
| Specification version | 0.2.0 |
| Specification revision (pin) | Source commit **`4445294`** ("Approve Tier B batch 2 after human-loop review"); Tier B batch 2; recorded in `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md` batch-4. Verified byte-identical between `4445294` and source HEAD (`37087db`, adds only unrelated batch-3 drafts). Content also covered by `SHA256SUMS` in the import directory. |
| Review status | Approved 2026-07-22 (source `docs/review/2026-07-22-tier-b-batch2-human-loop.md`, "all 12 approved") |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |

## Relied-upon claims

| claim_id | category | normative | backing observations |
|---|---|---|---|
| append.signature-primary | public-interface-fact | yes | `strings` |
| append.concatenates-text | black-box-observation | yes | `strings`, `multiple`, `char` |

Both claims are `status: confirmed` in the export's `provenance.yaml`.

## Observed cases (from `observations/append_observations.json`)

| case_id | call | class | size | value |
|---|---|---|---|---|
| `strings` | `append("a","b")` | string | `[1, 1]` | `"ab"` |
| `multiple` | `append("x","y","z")` | string | `[1, 1]` | `"xyz"` |
| `char` | `append('a','b')` | char | `[1, 2]` | `'ab'` |

## Unresolved questions carried

- `append.q-release` — introducing release; RunMat implements unconditionally.
- `append.q-types` — output type over the full char/string/mixed matrix; only
  the observed subset is asserted normative.

## Documented independent choices (non-normative, no approved source)

Trailing-whitespace preservation on char inputs; element-wise string-array
concatenation with implicit expansion; single-argument identity; empty-array
propagation; rejection of non-text inputs, broadcast-incompatible shapes, and
zero arguments. None are asserted as MATLAB-conformant; see `spec.md`
"Unresolved behaviour".

## Clean-room attestation

No MATLAB was invoked for this feature. All normative behaviour derives solely
from the approved `append` 0.2.0 export; unobserved behaviour is derived
independently from RunMat's architecture and marked as such. No MathWorks
source, documentation prose, examples, or raw observation logs entered this
repository.
