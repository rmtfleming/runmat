# Provenance Record — 021-psi

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `psi` 0.2.0 — `status: approved` |
| Specification revision | Commit pin: `e46587ae1b78237e0f43cd55f678de01a28495a4` of `matlab-interface-spec` (Tier B batch 1; see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, batch-3 entry) |
| Review status | Approved 2026-07-22 by repository maintainer (source `docs/review/2026-07-22-tier-b-batch1-human-loop.md`, "all 12 approved") |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

Imported artefacts (byte-identical to source at the pinned commit, per the
import manifest):
`specs/imports/matlab-interface-spec/specifications/psi/{interface.yaml,
behaviour.md, provenance.yaml, observations/psi_observations.json}`.

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| psi.signature-primary | public-interface-fact | yes |
| psi.output-class | black-box-observation | yes |
| psi.digamma-values | black-box-observation | yes |

Backing observations (2 cases, both `errored: false`, class `double`):
`scalar` — `psi(1)` → `-0.57721566490153231`, size 1×1;
`vector` — `psi([1 2 3])` →
`[-0.57721566490153231 0.42278433509846747 0.92278433509846747]`, size 1×3.
Reproduction tolerance in tests: `1e-12` (floating-point observations).

## External algorithm source (Principle X)

- Algorithm: standard digamma evaluation — argument-shift recurrence
  ψ(x+1) = ψ(x) + 1/x followed by the Bernoulli-number asymptotic expansion
  ψ(x) ≈ ln x − 1/(2x) − Σ_{k≥1} B_{2k}/(2k·x^{2k}) for large x.
- Source: M. Abramowitz and I. A. Stegun (eds.), *Handbook of Mathematical
  Functions*, National Bureau of Standards Applied Mathematics Series 55
  (1964), equations 6.3.5 (recurrence) and 6.3.18 (asymptotic expansion).
  As a U.S. federal government work this handbook is in the public domain;
  no licence restrictions apply. The same classical scheme is described in
  J. M. Bernardo, "Algorithm AS 103: Psi (Digamma) Function", *Applied
  Statistics* 25(3), 1976 (consulted as a published description only; no
  code was copied).
- The Rust implementation was written from the mathematical formulas alone;
  no MathWorks source, documentation prose, or examples were consulted.
  Independent legal review MAY still be required (constitution Principle X).

## Independently derived inferences (not observations)

- Pole handling (`NaN` at non-positive integers), negative non-integer
  support, complex-input rejection, `NaN`/`Inf` propagation, integer/logical/
  char promotion, GPU gather-to-host, and rejection of the two-argument
  polygamma form are RunMat independent choices tagged `unresolved_choice_*`
  in tests. They are not presented as observed MATLAB behaviour.

Unresolved questions carried: `psi.q-polygamma` (two-argument polygamma
form), `psi.q-poles` (behaviour at non-positive integers).
