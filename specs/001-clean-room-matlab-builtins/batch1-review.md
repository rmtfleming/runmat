# Batch 1 Combined Review — Features 002, 003, 004 — 2026-07-22

Scope: F1 `002-type-predicates` (iscell, isstruct, isfile), F6 `003-path-parts`
(fileparts), F9 `004-minmax-bounds` (bounds). Cadence: approved batch-review
procedure (one human stop before implementation).

## Cross-artifact analysis (B3)

**Requirement → claim traceability**: 15 approved claims imported across the
5 functions; every normative FR cites ≥1 claim id; zero orphan claims
(every confirmed claim is used). Summary-derived requirements (isfile
existing→true; fileparts name/ext; bounds hi) are explicitly tagged — they
originate in approved summary text but have no observation case; their tests
carry `summary_derived` tags and their failure would not falsify an
observation.

**Requirement → test coverage**: each FR maps to named planned tests
(normative / summary_derived / unresolved_choice tiers). Every observed case
in the three exports' observation files appears in exactly one normative
test, using the exact recorded call inputs (e.g. `bounds([1 2; 3 4], 2)`).

**Task → requirement**: all tasks cite FRs; test tasks strictly precede
implementation tasks in every feature (Principle V).

**Provenance completeness**: three provenance.md files complete; all rest on
the interim content pin (`SHA256SUMS`, digest `311a11d7…daae`) under the
waiver recorded 2026-07-22. Ten unresolved questions carried explicitly; no
unresolved item generates a normative assertion.

**Scope & architectural impact**: 5 new files + mod-entry registrations; no
existing behaviour changed. One flagged possible exception: `bounds` may
need a `pub(crate)` visibility change in `min.rs`/`max.rs` helpers (surfaced
here per plan; falls within Principle IX reuse).

**Known specification thinness (accepted honestly)**: the exports confirm
only first outputs for `fileparts`/`bounds` and only the missing-path case
for `isfile`. The features implement complete, usable builtins with the
unconfirmed surface implemented as documented independent choices — never
asserted as MATLAB-conformant. Recommended follow-up to the specification
side: observation cases for `fileparts` outputs 2–3, `bounds` output 2,
`isfile` existing/folder cases, NaN options.

## Checklist

- [x] B1 requirements complete for 002/003/004 (specs + provenance)
- [x] B2 plans + tasks complete (exemplars: isnumeric.rs, fullfile.rs, max.rs)
- [x] B3 analysis above; zero traceability orphans; risks flagged
- [x] **BATCH GATE: human approval to implement (B4)** — APPROVED

## Approval record

- **2026-07-22** — maintainer approved batch implementation of features
  002, 003, 004 via interactive gate (all three features, full plans as
  written). Implementation (B4) authorised; publication (commit/push/PR)
  remains a separate approval.

## B4/B5 outcome (2026-07-22)

- Implemented: `iscell`, `isstruct` (`logical/tests/`), `isfile`,
  `fileparts` (`io/repl_fs/`), `bounds` (`math/reduction/`) — 5 new files +
  3 mod-entry registrations; no existing behaviour changed, no min/max
  visibility change needed.
- Validation: `cargo check` clean; `cargo fmt --check` clean; **26/26 tests
  pass** (13 normative — all 18 observed cases reproduced exactly; 3
  summary-derived; 8 unresolved-choice/validation). Per-feature evidence in
  each feature's `validation.md`.
- Publication (commit/push/PR): NOT performed — awaiting separate approval.
