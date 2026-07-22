# Provenance Record — 015-input-parsing

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `inputParser` 0.2.0 — `status: approved` |
| Specification revision | **Commit pin `b054f3ad809ceee28529a4dd7a6e28d2b6b434ef`** of `matlab-interface-spec`, via packaged export `dist/runmat-export-2026-07-22` (see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, batch 2 — authoritative pin; supersedes the batch-1 interim content pin) |
| Review status | Packaged-export review approved 2026-07-22 by repository maintainer (batch-2 record in `IMPORT-MANIFEST.md`; per-spec `status: approved` verified in `interface.yaml`) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |
| Import location | `specs/imports/matlab-interface-spec/specifications/inputParser/` (`interface.yaml`, `behaviour.md`, `provenance.yaml`, `observations/inputParser_observations.json` — 6 cases) |

## Relied-upon claims

| claim_id | category | normative | backing cases |
|---|---|---|---|
| inputParser.signature-primary | public-interface-fact | yes | construct-class |
| inputParser.results-fields | black-box-observation | yes | construct-class, results-class, required-value, param-value, param-default |
| inputParser.using-defaults | black-box-observation | yes | using-defaults, param-default |

## Source-category notes

- The two observation-backed claims are the ONLY normative behaviour for this
  feature. Everything else (method enumeration, handle-class statement, the
  `Unmatched` property, `addOptional`, validation functions, `KeepUnmatched`,
  `StructExpand`, case sensitivity, error/warning conditions) appears in the
  export only inside unresolved questions or `status: pending` entries and is
  therefore NON-NORMATIVE.
- The interface text "An inputParser object (handle)" (outputs entry) carries
  `status: pending`; handle semantics beyond the observed stateful call chains
  are treated as an independently derived implementation choice, not as an
  observation (see spec.md, Unresolved behaviour).

## Unresolved questions carried

- `inputParser.q-class-semantics` — handle-class/member scoping of the
  interface (methods addRequired, addOptional, addParameter, parse;
  properties Results, Unmatched, UsingDefaults) is not yet specified as
  individual observable claims.
- `inputParser.q-methods` — per-method observable behaviour (arguments,
  validation, errors) not yet enumerated on the specification side.
