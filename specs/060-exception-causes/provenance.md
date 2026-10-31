# Provenance Record — 060-exception-causes

| Field | `addCause` | `MException` (cause claim) | `throw` (cause claim) |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `addCause` **0.2.0**, `status: approved` | `MException` **0.2.0**, `status: approved` | `throw` **0.2.0**, `status: approved` |
| Import pin | **`fc9f80485f8b5df44a2a6b1ba6aec0e8274aa6c7`** (IMPORT-MANIFEST Batch 13, N4) | `8f72edb` (Batch 10, N2; imported for F49) | `1156c97` (Batch 11, N3; imported for F49) |
| Observations | 10 cases | case `addcause` | case `cause` |
| Review status | approved 2026-10-30 (source `docs/review/2026-10-30-n4-new-specs-human-loop.md`) | approved (see `specs/049-exceptions-warnings/provenance.md`) | approved (same) |

| Field | Value |
|---|---|
| Import date | 2026-10-30 |
| Selection | maintainer, 2026-10-30, `specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md` (F60; G1–G3 pre-approved; "Implement cause support", superseding D-049-1 "document only") |
| Freshness check (start) | 2026-10-30: in `../matlab-interface-spec`, after `git fetch -q origin`, HEAD = origin/main = `c3305a5` (tools only). `git log fc9f804..origin/main` over `specifications/{addCause,MException,throw}` was empty; `git status --short` showed no draft or experiment for these functions. |
| Freshness check (finish) | recorded in `validation.md` |
| Applicable release | R2026a (as recorded in each imported `provenance.yaml`) |
| Imported files | `specs/imports/matlab-interface-spec/specifications/addCause/{interface.yaml,behaviour.md,provenance.yaml,observations/addCause_observations.json}`; `MException/` and `throw/` as imported for F49 |

## Relied-upon claims

All are `source_category: black-box-observation`, `status: confirmed`.
Every id below exists in the imported `provenance.yaml` files (checked with
`grep claim_id`).

| claim_id | normative | used by |
|---|---|---|
| addCause.signature-primary | yes | FR-060-01 |
| addCause.append | yes | FR-060-01, FR-060-02 |
| addCause.nested | yes | FR-060-03 |
| addCause.value-semantics | yes | FR-060-04 |
| addCause.thrown | yes | FR-060-05 |
| addCause.caught-cause | yes | FR-060-06 |
| addCause.method-syntax | yes | FR-060-07 |
| addCause.error-char-cause | yes | FR-060-08 |
| addCause.non-exception | **no** (dispatch level) | U-060-1 (documented choice) |
| MException.addcause | yes | FR-060-01, FR-060-04 (closes FR-049-06) |
| throw.cause | yes | FR-060-05 (closes FR-049-06) |
| MException.properties | yes | FR-060-02 (0×1 cause cell without causes; already F49) |
| MException.rethrow | yes | basis for U-060-2 |

## Unresolved questions carried

- `addCause.q-warning-coverage` → FR-060-11 (not asserted).

## Clean-room statement

Only the imported exports listed above and RunMat source were used. MATLAB
was not invoked; no MathWorks documentation, source or examples were
consulted.
