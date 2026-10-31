# Provenance Record — 057-text-parsing

| Field | `strread` | `asManyOfPattern` |
|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same |
| Approved specification | `strread` **0.2.0**, `status: approved` | `asManyOfPattern` **0.2.0**, `status: approved` |
| Specification revision (import pin) | **`1156c975db1f393b58252a44e04333a130211398`** (IMPORT-MANIFEST Batch 11, N3) | **`fc9f80485f8b5df44a2a6b1ba6aec0e8274aa6c7`** (IMPORT-MANIFEST Batch 13, N4) |
| Review status | approved 2026-10-30 (source `docs/review/2026-10-30-n3-new-specs-human-loop.md`) | approved 2026-10-30 (source `docs/review/2026-10-30-n4-new-specs-human-loop.md`) |
| Observations | 15 cases | 14 cases |
| Applicable release | R2026a (GLNXA64), per the imported `provenance.yaml` | same |

| Field | Value (both) |
|---|---|
| Import date | 2026-10-30 |
| Selection | maintainer, 2026-10-30, `specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md` (F57; G1–G3 pre-approved) |
| Freshness check (start) | 2026-10-30: in `../matlab-interface-spec`, after `git fetch -q origin`, HEAD = origin/main = `c3305a5` (tools-only commit after `fc9f804`). `git log 1156c97..origin/main -- specifications/strread` and `git log fc9f804..origin/main -- specifications/asManyOfPattern` list nothing; `git status --short` for those spec and experiment paths is clean. Each imported file is byte-identical to `git show <pin>:specifications/<fn>/<file>`. |
| Freshness check (finish) | recorded in `validation.md` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/{strread,asManyOfPattern}/{interface.yaml,behaviour.md,provenance.yaml,observations/*_observations.json}` |

## Relied-upon claims

Every id below exists in the imported `provenance.yaml` (checked with
`grep claim_id`; 10 + 10 = 20 ids, all listed).

| claim_id | category | used by |
|---|---|---|
| strread.signature-primary | observation, normative | FR-057-01 |
| strread.default | observation, normative | FR-057-01 |
| strread.s | observation, normative | FR-057-02 |
| strread.numeric | observation, normative | FR-057-03 |
| strread.cyclic | observation, normative | FR-057-04 |
| strread.delimiter | observation, normative | FR-057-05 |
| strread.empty | observation, normative | FR-057-06 |
| strread.no-warning | observation, normative | FR-057-09 |
| strread.error-numeric | observation, normative | FR-057-07 |
| strread.error-bad-format | observation, normative | FR-057-08 |
| asManyOfPattern.signature-primary | observation, normative | FR-057-13 |
| asManyOfPattern.class | observation, normative | FR-057-13 |
| asManyOfPattern.cobra-pattern | observation, normative | FR-057-14 |
| asManyOfPattern.greedy | observation, normative | FR-057-16 |
| asManyOfPattern.min-max | observation, normative | FR-057-17 |
| asManyOfPattern.text-pattern | observation, normative | FR-057-18 |
| asManyOfPattern.empty-matches | observation, **non-normative** (pending `q-empty-match`) | FR-057-27 |
| asManyOfPattern.error-min-gt-max | observation, normative | FR-057-19 |
| asManyOfPattern.error-negative | observation, normative | FR-057-21 |
| asManyOfPattern.error-numeric | observation, normative | FR-057-22 |

## Unresolved questions carried

`strread.q-warning-coverage`, `strread.q-more-options`,
`strread.q-error-ids`, `asManyOfPattern.q-warning-coverage`,
`asManyOfPattern.q-empty-match`. `spec.md` ("Unresolved behaviour") records
how each is handled.

## Source categories and independence

- FR-057-01..09, 13, 14, 16..19, 21, 22 come from black-box observations;
  the identifiers are raised in the RunMat namespace (D-057-1).
- FR-057-25..31 are **independently derived documented choices** or
  explicit `UnsupportedOption` dispositions. Tests tag them
  `documented_choice_*` / `unresolved_*`.
- Algorithms, written for this feature:
  - strread: split on whitespace or on one delimiter character, then assign
    fields to conversions round-robin; numbers parsed with Rust's `f64`
    parser.
  - asManyOfPattern: wrap the pattern's regular expression in a greedy
    quantifier (`regex` crate syntax), using RunMat's existing pattern
    object helpers.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
