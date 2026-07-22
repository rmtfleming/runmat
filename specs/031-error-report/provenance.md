# Provenance Record — 031-error-report

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `getReport` 0.2.0 — `status: approved` |
| Specification revision | Commit pin: `1b019121bb747e63d69a3eb005534cb60669e5f4` ("Clean up stale SKELETON labels"; approved at `67eaf83`; Tier B batch 3 import — see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, batch-5 section) |
| Review status | Approved 2026-07-22 by repository maintainer (source `docs/review/2026-07-22-tier-b-batch3-human-loop.md`, "all 12 approved"; F31 authorised in `specs/001-clean-room-matlab-builtins/selection-proposal-tierb3.md`) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| getReport.signature-primary | public-interface-fact | yes |
| getReport.output-class | black-box-observation | yes |
| getReport.char-report | black-box-observation | yes |

The single observation case (`basic` → `getReport(MException('x:y','oops'))`
= `'oops'`, 1×4 char) was captured on R2026a GLNXA64. It establishes that the
report of a plain `MException` is its message text, returned as a char row
vector. The fuller MATLAB report (identifier formatting, stack trace, and the
`'extended'`/`'basic'`/hyperlink options) is not observed; those behaviours
are documented independent choices (no observation backing) and are tagged as
such in the feature spec and tests.

## Unresolved questions carried

- `getReport.q-options` — the `'extended'`/`'basic'` format selectors and
  hyperlink handling are not exercised; the feature omits them (Fixed single
  output) and returns the message-only report for every `MException` until the
  specification side observes the extended format.
