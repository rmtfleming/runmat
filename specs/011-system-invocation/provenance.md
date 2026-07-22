# Provenance Record — 011-system-invocation

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `system` 0.2.0 — `status: approved` |
| Specification revision | Commit `b054f3ad809ceee28529a4dd7a6e28d2b6b434ef` of `matlab-interface-spec`, imported via packaged export `dist/runmat-export-2026-07-22` (see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, batch-2 section — authoritative pin; supersedes the batch-1 interim content pin) |
| Review status | Approved 2026-07-22 by repository maintainer (packaged export, batch-2 import) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

Backing observations: `specs/imports/matlab-interface-spec/specifications/`
`system/observations/system_observations.json` — 3 independently constructed
cases on R2026a (GLNXA64): `status-success`, `status-failure`,
`captured-output`.

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| system.signature-primary | public-interface-fact | yes |
| system.status-and-output | black-box-observation | yes |

Unresolved questions carried: `system.q-side-effect` (external-process side
effects; platform dependence), `system.q-outputs` (`cmdout` text encoding;
range of nonzero exit codes across platforms).

Note: in the imported `interface.yaml`, the per-parameter entries (`command`
input, `status`/`cmdout` outputs) carry `status: pending`; the invocation
forms and the `system.status-and-output-rule` shape rule are `confirmed`.
Normative requirements in `spec.md` rely only on the confirmed claims above.
