# Provenance Record — 012-url-fetch

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifier | `urlread` 0.2.0, `status: approved` |
| Specification revision | **Commit pin `b054f3ad809ceee28529a4dd7a6e28d2b6b434ef`** of `matlab-interface-spec`, imported via packaged export `dist/runmat-export-2026-07-22` (see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, batch-2 section; supersedes the batch-1 interim content pin) |
| Review status | Approved 2026-07-22 by repository maintainer (packaged-export / human-loop review, batch 2) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) |

Import artefacts:
`specs/imports/matlab-interface-spec/specifications/urlread/{interface.yaml,behaviour.md,provenance.yaml,observations/urlread_observations.json}`
(1 observation case, `file-url`; drift check via
`specs/imports/matlab-interface-spec/SHA256SUMS`).

## Relied-upon claims

| claim_id | category | normative |
|---|---|---|
| urlread.signature-primary | public-interface-fact | yes |
| urlread.returns-char | black-box-observation | yes |
| (summary text: "Read the contents at a URL and return the result as text" — basis of the http/https path, FR-012-03) | approved summary, no observation | summary-derived |

The sole observation case is `file-url` (local `file://` temp file →
`'hello urlread'`, char 1×13; NO external network was contacted, per the
observation file's notes). Both claims are backed only by that case.

## Unresolved questions carried

- `urlread.q-network` — http/https network behaviour not exercised (network
  experiments need approval and a stable, permissible endpoint).
- `urlread.q-deprecated` — possible runtime deprecation notice and indicated
  replacement not exercised.

Additionally absent from the export (not even summary-derived; handled as
documented independent choices in `spec.md`): error conditions/identifiers,
any second output, charset rules, timeout behaviour, non-file/non-http
schemes.
