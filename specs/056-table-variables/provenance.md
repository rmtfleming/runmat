# Provenance Record — 056-table-variables

| Field | `renamevars` | `rows2vars` | `varfun` | `convertvars` |
|---|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same | same |
| Approved specification | `renamevars` **0.2.0**, `status: approved` | `rows2vars` **0.2.0**, `status: approved` | `varfun` **0.2.0**, `status: approved` | `convertvars` **0.2.0**, `status: approved` |
| Observations | 17 cases | 17 cases | 12 cases | 14 cases |
| Import pin | `1156c975db1f393b58252a44e04333a130211398` (IMPORT-MANIFEST Batch 11, N3) | `1156c97` (Batch 11) | `fc9f80485f8b5df44a2a6b1ba6aec0e8274aa6c7` (Batch 13, N4) | `fc9f804…` (Batch 13) |
| Review status | approved 2026-10-30 (source `docs/review/2026-10-30-n3-new-specs-human-loop.md`) | same | approved 2026-10-30 (source `docs/review/2026-10-30-n4-new-specs-human-loop.md`) | same |

| Field | Value (all four) |
|---|---|
| Import date | 2026-10-30 |
| Selection | maintainer, 2026-10-30, `specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md` (F56; G1–G3 pre-approved, "implement N3 and N4") |
| Freshness check (start) | 2026-10-30: in `../matlab-interface-spec`, after `git fetch -q origin`, HEAD = origin/main = `c3305a5` (tools only, later than both pins). `git log 1156c97..origin/main` and `..HEAD` over `specifications/{renamevars,rows2vars}` were empty; over `specifications/{varfun,convertvars}` they list only `fc9f804` (the pin that added them). `git status --short` for the four spec and experiment directories was clean (no drafts). `diff -r` of each source directory against the imported copy: identical. |
| Freshness check (finish) | recorded in `validation.md` |
| Applicable release | R2026a (GLNXA64), as recorded in each imported `provenance.yaml` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/{renamevars,rows2vars,varfun,convertvars}/{interface.yaml,behaviour.md,provenance.yaml,observations/*_observations.json}` |

## Relied-upon claims

All are `source_category: black-box-observation`, `status: confirmed`.
The 41 normative claims back requirements; the 4 `*.non-table` claims are
`normative: false` and are not requirements.

| claim_id | normative | used by |
|---|---|---|
| renamevars.signature-primary | yes | FR-056-01 |
| renamevars.selection | yes | FR-056-02 |
| renamevars.keeps-data | yes | FR-056-03 |
| renamevars.same-name | yes | FR-056-03 |
| renamevars.swap | yes | FR-056-03 |
| renamevars.non-identifier | yes | FR-056-03 |
| renamevars.error-duplicate | yes | FR-056-04 |
| renamevars.error-unknown | yes | FR-056-04 |
| renamevars.error-count | yes | FR-056-04 |
| renamevars.error-empty-name | yes | FR-056-04 |
| renamevars.error-index-range | yes | FR-056-04 |
| renamevars.error-dim-Row | yes | FR-056-04 |
| renamevars.non-table | no | (FR-056-40 choice only) |
| rows2vars.signature-primary | yes | FR-056-10 |
| rows2vars.layout | yes | FR-056-10 |
| rows2vars.default-names | yes | FR-056-11 |
| rows2vars.names-source | yes | FR-056-12 |
| rows2vars.naming-modify | yes | FR-056-13 |
| rows2vars.warning-warn-naming-default | yes | FR-056-13 |
| rows2vars.naming-preserve | yes | FR-056-13 |
| rows2vars.duplicate-source | yes | FR-056-14 |
| rows2vars.mixed | yes | FR-056-15 |
| rows2vars.empty-rows | yes | FR-056-16 |
| rows2vars.error-multicol | yes | FR-056-17 |
| rows2vars.error-unknown-source | yes | FR-056-17 |
| rows2vars.error-bad-rule | yes | FR-056-17 |
| rows2vars.non-table | no | (FR-056-40 choice only) |
| varfun.signature-primary | yes | FR-056-20 |
| varfun.names | yes | FR-056-20 |
| varfun.rows | yes | FR-056-21 |
| varfun.cobra-pattern | yes | FR-056-21 |
| varfun.rownames | yes | FR-056-22 |
| varfun.empty-rows | yes | FR-056-22 |
| varfun.error-cellstr | yes | FR-056-23 |
| varfun.error-not-function | yes | FR-056-23 |
| varfun.non-table | no | (FR-056-40 choice only) |
| convertvars.signature-primary | yes | FR-056-30 |
| convertvars.selection | yes | FR-056-30 |
| convertvars.cobra-form | yes | FR-056-30 |
| convertvars.types | yes | FR-056-31 |
| convertvars.function | yes | FR-056-32 |
| convertvars.error-unknown | yes | FR-056-33 |
| convertvars.error-bad-type | yes | FR-056-33 |
| convertvars.error-cell-to-double | yes | FR-056-33 |
| convertvars.non-table | no | (FR-056-40 choice only) |

## Unresolved questions carried

`renamevars.q-warning-coverage`, `renamevars.q-timetable`,
`renamevars.q-row-variable` (non-normative; documented choice FR-056-43),
`rows2vars.q-warning-coverage`, `rows2vars.q-more`,
`rows2vars.q-rownames-dup-warning`, `varfun.q-warning-coverage`,
`varfun.q-more-args`, `convertvars.q-warning-coverage`. `spec.md`
FR-056-40..53 records how each is handled.

## Source categories and independence

- FR-056-01..33 come from black-box observations. Observed identifiers are
  in the `MATLAB:` namespace and RunMat raises them in its own namespace.
- FR-056-40..53 are **independently derived documented choices** or
  explicit `UnsupportedOption` dispositions; tests tag them
  `documented_choice_*` / `unresolved_*` / `rejects_*`.
- Algorithms, written for this feature over RunMat's own table model:
  renaming is a key substitution in the variable store; `rows2vars` is a
  plain transpose of single-column variables; `varfun` invokes the handle
  through RunMat's `MappedCallable` (the mechanism `cellfun` uses);
  `convertvars` delegates to RunMat's existing conversion builtins by name
  through `call_builtin_async` (as F50 `cast`). The `rows2vars` name rule
  (FR-056-46) was derived only from the two observed input/output pairs.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
