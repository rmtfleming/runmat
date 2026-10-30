# Provenance Record — 046-isprop

| Field | `isprop` |
|---|---|
| Source repository | `matlab-interface-spec` (local) |
| Approved specification | `isprop` **0.2.0**, `status: approved` |
| Specification revision (import pin) | **`c65e2382fa2420416439f8342333af77dcaee5b3`** (IMPORT-MANIFEST Batch 9, N1) |
| Review status | approved 2026-10-29 (source `docs/review/2026-10-29-n1-new-specs-human-loop.md`, "Approve all 14") |
| Import date | 2026-10-29 |
| Selection | maintainer, 2026-10-30, `specs/001-clean-room-matlab-builtins/selection-proposal-n1.md` (F46; G1–G3 pre-approved) |
| Freshness check (start) | 2026-10-30: in `../matlab-interface-spec`, after `git fetch -q --all`, HEAD = origin/main = `8f72edb`. `git diff --name-only c65e238 HEAD` and the same against `origin/main`, both over `specifications/isprop`, returned nothing; `git status --short` for that path was clean. |
| Freshness check (finish) | recorded in `validation.md` |
| Applicable release | as recorded in the imported `provenance.yaml` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/isprop/{interface.yaml,behaviour.md,provenance.yaml,observations/isprop_observations.json}` (11 cases) |

## Relied-upon claims

All 7 claim ids exist in the imported `provenance.yaml` (checked in
`validation.md`). All are `black-box-observation`, `normative: true`.

| claim_id | used by |
|---|---|
| isprop.signature-primary | FR-046-01 |
| isprop.output-class | FR-046-01 |
| isprop.objects | FR-046-02 |
| isprop.case-insensitive-graphics | FR-046-03 |
| isprop.non-objects | FR-046-04 |
| isprop.arrays | FR-046-05 |
| isprop.string-name | FR-046-06 |

## Unresolved questions carried

`isprop.q-warning-coverage` and `isprop.q-case-objects`. `spec.md` records
how each is handled.

## Source categories and independence

- FR-046-01..06 come from observations. The export records no errors.
- FR-046-07..12 are **independently derived documented choices** or
  explicit `UnsupportedOption` dispositions. Tests tag them
  `documented_choice_*` / `unresolved_*`.
- The implementation only reuses RunMat's own lookups:
  - the plotting handle resolver and property getter
    (`builtins/plotting/core/properties.rs`);
  - the class registry, as used by the clean-room `properties` (spec 032);
  - `crate::builtins::table::is_table_value`.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
