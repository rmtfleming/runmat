# Provenance Record — 013-display-formatting

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `display` 0.2.0 — `status: approved` |
| Specification revision | **Commit pin: `b054f3ad809ceee28529a4dd7a6e28d2b6b434ef`** of `matlab-interface-spec`, imported via packaged export `dist/runmat-export-2026-07-22` (see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, Batch 2 — authoritative pin; supersedes the batch-1 interim content pin) |
| Review status | Approved — packaged-export review 2026-07-22 by repository maintainer (batch-2 record in `IMPORT-MANIFEST.md`; per-spec `status: approved` verified in `interface.yaml` at import) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| display.signature-primary | public-interface-fact | yes |
| display.formatted-text | black-box-observation | yes |

Backing observations: `specifications/display/observations/display_observations.json`,
cases `scalar-text`, `vector-text`, `char-text` (evalc-captured command-window
text; the observable is the emitted text, not a return value).

## Unresolved questions carried

- `display.q-side-effect` — command-window output is the primary effect; its
  capture interacts with formatting state; scope of specification open.
- `display.q-overload` — `display` is overloadable per class; which classes to
  specify is open.
- `display.q-return` — `display`'s own return value (if any) was not observed;
  the recorded char rows are evalc-captured text, not a return value.

Additional export-status notes relied on as *non-normative* context:

- Invocation form `display(X, name)` is listed `status: pending` in
  `interface.yaml` — NOT approved; any handling in RunMat is a documented
  independent choice.
- The `(none)` output row in `interface.yaml` is `status: pending` — the
  no-output-argument claim is not normative (see `display.q-return`).
