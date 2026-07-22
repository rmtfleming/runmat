# Provenance Record — 022-legacy-predicates

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote recorded) |
| Approved specification identifiers | `isobject` 0.2.0, `isstr` 0.2.0, `isfolder` 0.2.0, `isdir` 0.2.0 — all `status: approved` in their `interface.yaml` |
| Specification revision (pin) | Source commit **`4445294`** ("Approve Tier B batch 2 after human-loop review"); Tier B batch 2. See `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, Batch 4 section. Verified byte-identical between `4445294` and source HEAD (`37087db`). |
| Review status | Approved 2026-07-22 by repository maintainer (source `docs/review/2026-07-22-tier-b-batch2-human-loop.md`, "all 12 approved"). |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |

## Relied-upon claims

| claim_id | category | normative | backing observation case(s) |
|---|---|---|---|
| isobject.signature-primary | public-interface-fact | yes | object |
| isobject.output-class | black-box-observation | yes | object, numeric, char, cell |
| isobject.logical-object-test | black-box-observation | yes | object, numeric, char, cell |
| isstr.signature-primary | public-interface-fact | yes | char |
| isstr.output-class | black-box-observation | yes | char, numeric, string, cell |
| isstr.logical-char-test | black-box-observation | yes | char, numeric, string, cell |
| isfolder.signature-primary | public-interface-fact | yes | existing |
| isfolder.output-class | black-box-observation | yes | existing, missing |
| isfolder.logical-folder-test | black-box-observation | yes | existing, missing |
| isdir.signature-primary | public-interface-fact | yes | existing |
| isdir.output-class | black-box-observation | yes | existing, missing |
| isdir.logical-folder-test | black-box-observation | yes | existing, missing |

All relied-upon claims are `status: confirmed` and backed by cited black-box
observations or public-interface facts. No claim in this feature is derived
from MathWorks source, documentation prose, or examples. MATLAB was NOT invoked
in this repository.

## Source categories

- **black-box-observation**: the observed value/class/size results in each
  export's `observations/*.json`.
- **public-interface-fact**: the accepted primary invocation form
  (`tf = fn(A)` / `tf = fn(path)`).
- **independently-derived-inference**: none relied upon as normative; all
  behaviour beyond the observations is recorded as an unresolved documented
  choice (see below), not asserted as MATLAB-conformant.

## Unresolved questions carried

Carried forward from the exports into `spec.md`, tests (`unresolved_choice_*`),
and implementation doc-comments:

- `isobject.q-fundamental` — which built-in classes count as non-objects.
- `isstr.q-deprecated` — equivalence to `ischar` and any deprecation notice.
- `isfolder.q-file` — behaviour when the path is a file rather than a folder.
- `isdir.q-deprecated` — equivalence to `isfolder` and any deprecation notice.

## Licensing

No external implementation code was transferred. The implementation is derived
from RunMat's own `Value` model and its existing filesystem abstraction
(`runmat_filesystem`), mirroring the in-repo predicates `ischar` and `isfile`.
Independent legal review MAY still be required; this record does not substitute
for legal advice.
