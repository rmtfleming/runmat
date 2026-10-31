# Provenance Record — 054-types-structs-text

| Field | `isfloat` | `struct2cell` | `strvcat` | `isnat` |
|---|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same | same |
| Approved specification | **0.2.0**, `status: approved` | **0.2.0** | **0.2.0** | **0.2.0** |
| Batch | N3 | N3 | N3 | N4 |
| Import pin | `1156c975db1f393b58252a44e04333a130211398` (IMPORT-MANIFEST Batch 11) | same | same | `fc9f80485f8b5df44a2a6b1ba6aec0e8274aa6c7` (IMPORT-MANIFEST Batch 13) |
| Review record (spec repo) | `docs/review/2026-10-30-n3-new-specs-human-loop.md` | same | same | `docs/review/2026-10-30-n4-new-specs-human-loop.md` |
| Observations | 17 cases | 10 cases | 14 cases | 9 cases |

| Field | Value (all four) |
|---|---|
| Review status | approved 2026-10-30 after human-loop review |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64), as recorded in each imported `provenance.yaml` |
| Import date | 2026-10-30 |
| Selection | maintainer, 2026-10-30, `specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md` (F54; G1–G3 pre-approved) |
| Freshness check (start) | 2026-10-30: in `../matlab-interface-spec`, after `git fetch -q origin`, HEAD = origin/main = `1c91aa1` (batch N5, later than both pins). `git log 1156c97..origin/main` over `specifications/{isfloat,struct2cell,strvcat}` lists only `fc9f804`, whose diff does not touch those directories; `git log fc9f804..origin/main` (and `..HEAD`) over `specifications/isnat` is empty. `git status --short` shows no draft for any of the four; `diff -rq` of the imported directories against the spec repo's working tree is empty. |
| Freshness check (finish) | recorded in `validation.md` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/{isfloat,struct2cell,strvcat,isnat}/{interface.yaml,behaviour.md,provenance.yaml,observations/*_observations.json}` |

## Relied-upon claims

24 normative claims (`source_category: black-box-observation`,
`normative: true`, `status: confirmed`) plus 2 non-normative ones
(`normative: false`) that are used only as the basis of documented choices.
Every id exists in the imported `provenance.yaml` (checked with
`grep claim_id`, see `validation.md`).

| claim_id | normative | used by |
|---|---|---|
| isfloat.signature-primary | yes | FR-054-01 |
| isfloat.true-cases | yes | FR-054-02 |
| isfloat.false-cases | yes | FR-054-03 |
| isfloat.error-none | yes | FR-054-04 |
| isfloat.error-two | yes | FR-054-04 |
| isnat.signature-primary | yes | FR-054-05 |
| isnat.elementwise | yes | FR-054-06 |
| isnat.empty | yes | FR-054-07 |
| isnat.non-datetime | **no** | FR-054-25 (documented choice, D-054-2) |
| struct2cell.signature-primary | yes | FR-054-08 |
| struct2cell.scalar | yes | FR-054-09 |
| struct2cell.array-size | yes | FR-054-10 |
| struct2cell.array-order | yes | FR-054-11 |
| struct2cell.no-fields | yes | FR-054-12 |
| struct2cell.non-struct | **no** | FR-054-26 (documented choice, D-054-2) |
| strvcat.signature-primary | yes | FR-054-13 |
| strvcat.pad | yes | FR-054-13 |
| strvcat.skip-empty | yes | FR-054-14 |
| strvcat.cell | yes | FR-054-15 |
| strvcat.matrix-arg | yes | FR-054-16 |
| strvcat.numeric | yes | FR-054-17 |
| strvcat.string | yes | FR-054-18 |
| strvcat.empty | yes | FR-054-19 |
| strvcat.one | yes | FR-054-20 |
| strvcat.error-mixed | yes | FR-054-21 |
| strvcat.error-struct | yes | FR-054-22 |

## Unresolved questions carried

`isfloat.q-warning-coverage`, `isnat.q-warning-coverage`,
`struct2cell.q-warning-coverage` and `strvcat.q-warning-coverage`.
`spec.md` ("Unresolved behaviour") records how each is handled.

## Source categories and independence

- FR-054-01..22 come from black-box observations. The observed identifiers
  are in the `MATLAB:` namespace, and RunMat raises them in its own
  namespace (D-054-1).
- FR-054-23..28 are **independently derived documented choices** or
  explicit `UnsupportedOption` dispositions. Tests tag them
  `documented_choice_*` / `unresolved_*`.
- Implementation, all written for this feature or reused from RunMat:
  - isfloat: RunMat's own `class` name of the value (D-054-3).
  - isnat: `isnan` of the serial numbers in RunMat's datetime storage
    (D-054-4).
  - struct2cell: field-major copy of the struct array's field columns into a
    column-major cell, then the shape rule of FR-054-10.
  - strvcat: each input converted by RunMat's own `char` builtin (D-054-8),
    empty inputs dropped, rows stacked and padded with blanks.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
