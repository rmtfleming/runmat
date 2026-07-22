# Provenance Record — 032-object-introspection

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifiers | `properties` 0.2.0, `methods` 0.2.0, `what` 0.2.0 — all `status: approved` |
| Specification revision | Source commit pin **`1b019121bb747e63d69a3eb005534cb60669e5f4`** ("Clean up stale SKELETON labels"); Tier B batch 3 / `IMPORT-MANIFEST.md` batch-5. Approved at commit `67eaf83`; the only post-approval change is a cosmetic `notes` edit in the observation JSON (no case data/values/classes/claims changed). Byte-verified via `SHA256SUMS`. |
| Review status | Approved 2026-07-22 by repository maintainer (source `docs/review/2026-07-22-tier-b-batch3-human-loop.md`; all 12 batch specs approved, 6 minor pre-promotion corrections) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| properties.signature-primary | public-interface-fact | yes |
| properties.cellstr-names | black-box-observation | yes |
| methods.signature-primary | public-interface-fact | yes |
| methods.cellstr-names | black-box-observation | yes |
| what.signature-primary | public-interface-fact | yes |
| what.output-class | black-box-observation | yes |
| what.folder-struct | black-box-observation | yes |

`what.out-type` (interface `type_rule` on output `s`) is backed by the same
`what.output-class` observation and is treated as covered by FR-032-03.

## Observed cases (for reference only; not asserted verbatim)

| function | case | call | class | size |
|---|---|---|---|---|
| properties | object | `properties(inputParser)` | cell | 9×1 |
| properties | elem-type | `iscellstr(properties(inputParser))` | logical | 1×1 (true) |
| methods | object | `methods(inputParser)` | cell | 19×1 |
| methods | elem-type | `iscellstr(methods(inputParser))` | logical | 1×1 (true) |
| what | tempdir | `what(tempdir)` | struct | 1×1 |

The observed member counts (9, 19) and the concrete `what` struct contents
are MATLAB-object-/environment-specific and are **not** asserted — see
`spec.md` "Value non-assertion". Only class, shape, element type (for
`properties`/`methods`) and struct class/size/documented-field-presence (for
`what`) are normative.

## Unresolved questions carried

`properties.q-class-dependent`, `methods.q-class-dependent`, `methods.q-full`,
`what.q-fields`, `what.q-content` — recorded in `spec.md` "Unresolved
behaviour" and exercised by `unresolved_choice_*` tests.

## Clean-room attestation

No MATLAB was invoked in the production of this feature. No MathWorks source,
documentation prose, documentation examples, or raw observation logs entered
this repository; behavioural requirements derive solely from the approved
exports listed above. Source `provenance.yaml` author for these specs is
"clean-room agent (Claude Opus 4.8)"; recorded for traceability.
