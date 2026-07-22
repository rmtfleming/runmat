# Provenance Record — 024-histc

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `histc` 0.2.0 — `status: approved` |
| Specification version | 0.2.0 |
| Specification revision (commit pin) | `4445294` ("Approve Tier B batch 2 after human-loop review"); Tier B batch 2 / IMPORT-MANIFEST.md batch-4. Verified byte-identical between `4445294` and source HEAD `37087db` |
| Review status | Approved 2026-07-22 (source `docs/review/2026-07-22-tier-b-batch2-human-loop.md`, "all 12 approved", `histc` corrected spec-side before promotion) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |

## Relied-upon claims

| claim_id | category | normative | used by |
|---|---|---|---|
| histc.signature-primary | public-interface-fact | yes | FR-024-01 |
| histc.output-class | black-box-observation | yes | FR-024-01 |
| histc.bin-counts | black-box-observation | yes | FR-024-02, FR-024-03 |

Backing observation: `observations/histc_observations.json` case `basic` —
`histc([1 2 2 3], [1 2 3])` → value `[1 2 1]`, class `double`, size `[1, 3]`.

## Unresolved questions carried

- `histc.q-deprecated` — deprecation notice relative to `histcounts` (not
  implemented; no warning emitted).

## Documented independent choices (no approved backing; not conformance claims)

- Output orientation follows the operating dimension (column input → column
  output). *(FR-024-04)*
- Values below `edges(1)`, values strictly above `edges(end)`, and `NaN` are
  not counted. *(FR-024-04)*
- Matrix / N-D inputs are histogrammed along the first non-singleton dimension
  by default; optional third argument `dim` selects the dimension. *(FR-024-05)*
- Second output `idx` gives the 1-based bin index per element, `0` if
  uncounted. *(FR-024-06)*
- Non-monotonic edges raise an error. *(FR-024-07)*

## Clean-room attestation

No MATLAB was invoked in this feature. No MathWorks source, documentation
prose, documentation examples, or raw observation logs entered this repository.
The generalised bin rule (FR-024-03) is an independent inference consistent
with the single approved observation; the observation itself confirms only the
`[1 2 2 3]/[1 2 3] → [1 2 1]` case (including the final-bin/last-edge behaviour).
