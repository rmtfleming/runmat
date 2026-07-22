# Provenance Record — 008-date-formatting

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `datestr` 0.2.0 — `status: approved` |
| Specification revision | Interim content pin: `specs/imports/matlab-interface-spec/SHA256SUMS` (digest `311a11d7…daae`); commit pin pending source-repo commit (waiver recorded 2026-07-22) |
| Review status | Approved 2026-07-22 by repository maintainer (Tier A submission) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| datestr.signature-primary | public-interface-fact | yes |
| datestr.output-class | black-box-observation | yes |
| datestr.char-format | black-box-observation | yes |

Unresolved questions carried: `datestr.q-deprecated`, `datestr.q-locale`.

## External implementation sources (Principle X)

- Days-to-civil date conversion: independently implemented from the publicly
  documented civil-from-days algorithm (Howard Hinnant, "chrono-Compatible
  Low-Level Date Algorithms", released by its author into the public
  domain). No MathWorks material involved; no GPL code copied. Epoch offset
  (serial day 719529 = 1970-01-01) derived solely from the observed case
  `serial-default` (738885 → 30-Dec-2022).
