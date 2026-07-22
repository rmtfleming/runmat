# Provenance Record — 028-eps-function

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `eps` 0.2.0 — `status: approved` |
| Specification revision (pin) | Source commit **`1b019121bb747e63d69a3eb005534cb60669e5f4`** (Tier B batch 3; `IMPORT-MANIFEST.md` Batch 5). Approved at source commit `67eaf83`; the only post-approval change to these specs is a cosmetic `notes` edit in the observation JSON (no case data, values, classes, or claims changed) — still `status: approved` at HEAD. Content-verified additionally by `specs/imports/matlab-interface-spec/SHA256SUMS`. |
| Review status | Approved 2026-07-22 (source `docs/review/2026-07-22-tier-b-batch3-human-loop.md`; all 12 batch-3 specs approved) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |

## Relied-upon claims

| claim_id | category | normative | backing case(s) |
|---|---|---|---|
| eps.signature-primary | public-interface-fact | yes | `default` |
| eps.spacing-constants | black-box-observation | yes | `default`, `single-class` |

Observed cases (from `observations/eps_observations.json`):

| case | call | class | size | value |
|---|---|---|---|---|
| `default` | `eps` | double | `[1,1]` | `2.2204460492503131e-16` |
| `single-class` | `eps('single')` | single | `[1,1]` | `1.1920928955078125e-07` |

## Unresolved questions carried

- `eps.q-value-form` — the `eps(x)` value-dependent form is not exercised in
  the export. Carried into `spec.md` (Unresolved behaviour), the implementation
  (documented-choice `eps(x)` / `eps('double')` / array / complex / edge
  cases), and the tests (`unresolved_choice_*`).

## Source categories

- `eps.signature-primary` — public-interface fact (the `d = eps` form is
  accepted).
- `eps.spacing-constants` — black-box observation (the two exact values above).
- All other implemented behaviour (`eps(x)`, `eps('double')`, arrays, complex,
  negative, subnormal/Inf/NaN edges, error cases) — independently derived
  inference (documented choice), NOT presented as observation.

## Clean-room compliance

- Behaviour derived solely from the approved export + RunMat's own
  architecture. No MATLAB was invoked in this repository; no MathWorks source,
  documentation prose/examples, or raw observation logs were imported. The
  `eps(x)` spacing algorithm is the standard IEEE `nextUp`-based ulp
  computation (public numerical technique), not an inferred MATLAB internal.
