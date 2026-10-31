# Provenance Record — 053-environment

| Field | `isunix` | `ismac` | `isMATLABReleaseOlderThan` | `chdir` | `date` |
|---|---|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same | same | same |
| Approved specification | **0.2.0**, `status: approved` | **0.2.0** | **0.2.0** | **0.2.0** | **0.2.0** |
| Batch | N4 | N4 | N4 | N3 | N3 |
| Import pin | `fc9f80485f8b5df44a2a6b1ba6aec0e8274aa6c7` (IMPORT-MANIFEST Batch 13) | same | same | `1156c975db1f393b58252a44e04333a130211398` (IMPORT-MANIFEST Batch 11) | same |
| Review record (spec repo) | `docs/review/2026-10-30-n4-new-specs-human-loop.md` | same | same | `docs/review/2026-10-30-n3-new-specs-human-loop.md` | same |
| Observations | 4 cases | 2 cases | 10 cases | 8 cases | 5 cases |

| Field | Value (all five) |
|---|---|
| Review status | approved 2026-10-30 after human-loop review |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64), as recorded in each imported `provenance.yaml` |
| Import date | 2026-10-30 |
| Selection | maintainer, 2026-10-30, `specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md` (F53; G1–G3 pre-approved) |
| Freshness check (start) | 2026-10-30: in `../matlab-interface-spec`, after `git fetch -q origin`, HEAD = origin/main = `1c91aa1` (batch N5, later than both pins). `git log 1156c97..origin/main` over `specifications/{chdir,date}` lists only `fc9f804`, whose diff does not touch those two directories; `git log fc9f804..origin/main` (and `..HEAD`) over `specifications/{isunix,ismac,isMATLABReleaseOlderThan}` is empty. `git status --short` shows no draft for any of the five; `diff -rq` of the imported directories against the spec repo's working tree is empty. |
| Freshness check (finish) | recorded in `validation.md` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/{isunix,ismac,isMATLABReleaseOlderThan,chdir,date}/{interface.yaml,behaviour.md,provenance.yaml,observations/*_observations.json}` |

## Relied-upon claims

All 26 are `source_category: black-box-observation`, `normative: true`,
`status: confirmed`. Every id exists in the imported `provenance.yaml`
(checked with `grep claim_id`, see `validation.md`).

| claim_id | used by |
|---|---|
| isunix.signature-primary | FR-053-01 |
| isunix.value | FR-053-01 |
| isunix.cobra-test | FR-053-03 |
| isunix.error-arg | FR-053-04 |
| ismac.signature-primary | FR-053-02 |
| ismac.value | FR-053-02 |
| ismac.error-arg | FR-053-04 |
| isMATLABReleaseOlderThan.signature-primary | FR-053-05 |
| isMATLABReleaseOlderThan.comparison | FR-053-06 |
| isMATLABReleaseOlderThan.text | FR-053-07 |
| isMATLABReleaseOlderThan.error-bad-name | FR-053-08 |
| isMATLABReleaseOlderThan.error-numeric | FR-053-09 |
| isMATLABReleaseOlderThan.error-none | FR-053-10 |
| chdir.signature-primary | FR-053-11 |
| chdir.changes | FR-053-11 |
| chdir.trailing-sep | FR-053-12 |
| chdir.relative | FR-053-13 |
| chdir.output-old | FR-053-14 |
| chdir.no-arg | FR-053-15 |
| chdir.error-missing | FR-053-16 |
| chdir.error-numeric | FR-053-17 |
| date.signature-primary | FR-053-18 |
| date.output-class | FR-053-18 |
| date.format | FR-053-19 |
| date.agrees-clock | FR-053-20 |
| date.error-arg | FR-053-21 |

## Unresolved questions carried

`isunix.q-platform`, `ismac.q-platform`,
`isMATLABReleaseOlderThan.q-release`, `chdir.q-display`, `date.q-zero-pad`,
`date.q-other-months`, `date.q-locale`, and the five
`*.q-warning-coverage`. `spec.md` ("Unresolved behaviour") records how each
is handled.

## Source categories and independence

- FR-053-01..21 come from black-box observations. The observed identifiers
  are in the `MATLAB:` namespace, and RunMat raises them in its own
  namespace (D-053-1). FR-053-01..03 hold on a Linux host, the only platform
  observed.
- FR-053-22..27 are **independently derived documented choices** or explicit
  `UnsupportedOption` dispositions. Tests tag them `documented_choice_*` /
  `unresolved_*`.
- Implementation, all written for this feature or reused from RunMat:
  - isunix/ismac: compile-time `cfg!` platform tests.
  - isMATLABReleaseOlderThan: a small parser for `R<year><a|b>` and a
    comparison with RunMat's own compatibility pin (D-053-3).
  - chdir: argument checks, then RunMat's `cd` implementation (D-053-4).
  - date: RunMat's `clock` builtin plus a fixed English month table
    (D-053-5).
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
