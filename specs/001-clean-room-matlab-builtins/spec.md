# Feature Specification: Clean-Room MATLAB Builtins Programme Governance

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit feature id: `001-clean-room-matlab-builtins`; export `SPECIFY_FEATURE=001-clean-room-matlab-builtins` when running SpecKit scripts)

**Created**: 2026-07-21

**Status**: Draft — awaiting Gate 1 human approval; implementation BLOCKED (see Dependencies)

**Input**: User description: "Establish the governed workflow by which approved,
implementation-neutral MATLAB interface specifications from the
matlab-interface-spec repository enter RunMat and drive independent Rust
implementations of MATLAB-compatible built-in functions. This feature defines
the process; it implements no built-in."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Import an approved specification export (Priority: P1)

A RunMat maintainer takes an approved interface specification export from
`matlab-interface-spec` and bootstraps a governed SpecKit feature for one
built-in, with provenance recorded and implementation blocked until the gates
are passed.

**Why this priority**: This is the entry point of the entire programme; no
built-in work is legitimate without it.

**Independent Test**: Can be tested by bootstrapping a feature from a candidate
export and verifying that the feature directory contains a complete provenance
record, that every normative requirement cites an approved claim identifier,
and that implementation tasks are refused while any gate is unapproved.

**Acceptance Scenarios**:

1. **Given** an eligible approved export (per FR-002), **When** a maintainer
   bootstraps a feature from it, **Then** the feature directory contains
   `spec.md` and `provenance.md` with all mandatory provenance fields
   (FR-003) and the feature stops at Gate 1.
2. **Given** an export that lacks a citable committed revision, **When** a
   maintainer attempts to bootstrap a feature from it, **Then** the feature is
   created in BLOCKED state and no plan, tasks or implementation may proceed.
3. **Given** an approved export revision recorded in a feature, **When** the
   export's source content later changes, **Then** the feature's provenance
   pin no longer matches and the mismatch is reported at the next gate or
   analysis run (drift detection), rather than silently absorbed.

---

### User Story 2 - Trace requirements to acceptance evidence (Priority: P2)

A reviewer at any gate can trace every normative behavioural requirement of a
built-in feature back to an approved specification claim and forward to
acceptance criteria and tests.

**Why this priority**: Traceability is what makes the clean-room defensible;
without it, approval gates cannot certify anything.

**Independent Test**: Can be tested on any bootstrapped feature by checking
that the requirement→claim→test mapping is total in both directions and that
the analysis phase flags any orphan requirement, orphan test, or unresolved
behaviour presented as resolved.

**Acceptance Scenarios**:

1. **Given** a feature specification with normative requirements, **When**
   cross-artifact analysis runs, **Then** every requirement maps to at least
   one acceptance criterion and (by implementation time) at least one test,
   and every implementation task maps back to a requirement.
2. **Given** a behaviour the approved export marks unresolved or non-normative,
   **When** the feature specification is drafted, **Then** that behaviour
   appears in an explicit "Unresolved behaviour" list and produces no
   normative requirement and no assertion-bearing test.

---

### User Story 3 - Select the first concrete built-in (Priority: P3)

The programme selects its first concrete built-in through a recorded,
human-approved selection process rather than by default or convenience.

**Why this priority**: Selection can only happen after the import and
traceability machinery exists; it is the first consumer of this governance
feature.

**Independent Test**: Can be tested by producing a selection proposal document
listing candidate exports with eligibility status and receiving an explicit
human selection decision at a gate.

**Acceptance Scenarios**:

1. **Given** one or more candidate exports in `matlab-interface-spec`,
   **When** the selection process runs, **Then** a proposal records for each
   candidate: eligibility per FR-002, estimated scope, affected RunMat crates,
   and any blockers, and a human selects the first built-in explicitly.
2. **Given** no eligible export exists, **When** the selection process runs,
   **Then** the proposal records the blocking dependency and no built-in
   feature is bootstrapped.

---

### Edge Cases

- Export approved but its repository has no commits: feature is BLOCKED at
  Gate 1 (this is the current state — see Dependencies).
- Export revision moves after import: drift must be reported, and the feature
  must either re-import at a new approved revision (returning to Gate 1) or
  keep its original pin.
- Raw observation files inside an export directory: they MUST NOT be copied
  into RunMat; only the independently worded specification artefacts named in
  FR-004 may be imported by reference or quotation of claim identifiers.
- A contributor or agent discovers MathWorks-derived content in a proposed
  input: the content is quarantined and the finding surfaced at the next gate.
- A behaviour needed for implementation is absent from the approved export:
  it is recorded as unresolved; the implementation must either avoid
  depending on it or the feature returns to the specification side for a new
  approved export revision.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001 (Two-repository boundary)**: The programme MUST treat
  `matlab-interface-spec` as the sole source of normative MATLAB behavioural
  requirements, with material flowing only from approved exports into RunMat
  features. RunMat tooling MUST NOT write to, and MUST NOT automatically read
  unreviewed material from, `matlab-interface-spec`.

- **FR-002 (Approved-export eligibility)**: An export is eligible for import
  only if all of the following hold: (a) it resides in
  `matlab-interface-spec` under `specifications/<function>/`; (b) it
  validates against that repository's published schemas; (c) its review
  status is approved with a named human reviewer; (d) it is identified by a
  citable committed revision (commit hash or tag) of
  `matlab-interface-spec`; and (e) the export consists of
  implementation-neutral artefacts [NEEDS CLARIFICATION: exact artefact set —
  see Q2 in Clarifications].

- **FR-003 (Provenance metadata)**: Every feature bootstrapped from an export
  MUST record in `provenance.md`: source repository; specification
  identifier; specification version or commit; review status and reviewer;
  import date; and the source category (black-box observation,
  public-interface fact, or independently derived inference) of every
  imported claim it relies on.

- **FR-004 (Import content restriction)**: Feature specifications MUST import
  only independently worded specification content and claim identifiers.
  Raw observation logs, experiment transcripts, MathWorks documentation prose
  and MathWorks examples MUST NOT be copied into this repository.

- **FR-005 (Feature-level traceability)**: Every normative requirement in a
  built-in feature MUST cite at least one approved claim identifier from the
  imported export; every acceptance criterion and test MUST reference the
  requirement(s) it verifies; every implementation task MUST reference the
  requirement(s) it serves. Analysis MUST fail the gate on any orphan in
  either direction.

- **FR-006 (Unresolved behaviour)**: Behaviour the export marks unresolved,
  non-normative, or hypothesis-grade MUST be listed in the feature under
  "Unresolved behaviour", MUST NOT generate normative requirements or
  behavioural assertions in tests, and MUST be preserved as unresolved in
  validation reports.

- **FR-007 (Acceptance-test structure)**: Each built-in feature MUST define,
  before implementation, acceptance tests organised to cover: returned
  values; result class/type; result shape; error conditions; warning
  conditions; and specification-named edge cases. Release-dependent behaviour
  MUST be tagged with the applicable release(s) from the export. Tests MUST
  live in RunMat's existing test layout for the affected crate(s), and no
  reported test outcome may be produced by anything other than an actual test
  run.

- **FR-008 (MATLAB invocation prohibition)**: No workflow, build step, test,
  script or agent action in this repository may invoke MATLAB in any form
  (process, Engine API, or MATLAB MCP tools). Compatibility evidence comes
  only from approved exports on the specification side.

- **FR-009 (Human approval gates)**: Each feature MUST stop for recorded
  human approval at Gate 1 (requirements), Gate 2 (plan and tasks) and Gate 3
  (analysis), and implementation approval MUST be separate from publication
  (commit/push/PR) approval. Gate decisions MUST be recorded in the feature's
  `agent-runs/` ledger.

- **FR-010 (First built-in selection)**: The first concrete built-in MUST be
  chosen via a selection proposal enumerating candidate exports with
  eligibility status (FR-002), scope estimate, affected crates and blockers,
  submitted for explicit human selection at a gate. No built-in feature may
  be bootstrapped before that decision.

- **FR-011 (Branch and feature identification)**: Programme branches are
  named `feature/<slug>`; because SpecKit feature detection otherwise relies
  on numbered branch names, SpecKit script invocations MUST set
  `SPECIFY_FEATURE=<NNN-slug>` (for this feature:
  `001-clean-room-matlab-builtins`).

### Key Entities

- **Approved export**: A reviewed, implementation-neutral specification for
  one built-in, identified by function name and a committed revision of
  `matlab-interface-spec`; carries claims with identifiers, source categories
  and review status.
- **Claim**: A single provenance-tracked statement about observable interface
  behaviour, categorised as black-box observation, public-interface fact, or
  independently derived inference (non-normative).
- **Builtin feature**: A SpecKit feature directory under `specs/` holding the
  specification, provenance record, plan, tasks, analysis results, gate
  ledger and validation evidence for one built-in (or tightly coupled
  family).
- **Gate record**: A ledger entry recording which human approved which gate,
  when, and with what conditions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of built-in features bootstrapped under this programme
  contain a complete provenance record (all FR-003 fields present) before
  Gate 1 review.
- **SC-002**: 100% of normative requirements in built-in features cite an
  approved claim identifier, and cross-artifact analysis reports zero orphan
  requirements, orphan tasks, or untested requirements at Gate 3.
- **SC-003**: Zero MATLAB invocations from this repository across the
  programme (verifiable by inspection of workflows, scripts and test code).
- **SC-004**: Every gate passage for every feature has a recorded human
  decision in the feature's ledger; zero implementation work precedes Gate 3
  approval.
- **SC-005**: Unresolved behaviours remain explicitly labelled unresolved in
  the final validation report of every feature that has any.

## Unresolved behaviour

None at programme level. (Per-built-in unresolved behaviours are recorded in
each built-in feature. This governance feature defines no MATLAB behaviour:
behavioural requirements for `median` or any other built-in are deliberately
absent and MUST NOT be added here.)

## Assumptions

- The `matlab-interface-spec` repository will gain committed, citable
  revisions and an explicit approval marking before any built-in feature
  proceeds past Gate 1.
- The human approver for gates is the repository owner (single-maintainer
  workflow) unless a different reviewer-of-record is designated (see Q4).
- RunMat's existing builtin architecture (`crates/runmat-runtime/src/builtins/`,
  `runtime_builtin` macro registration, `Value` runtime representation) is
  the implementation substrate; this feature introduces no new runtime
  architecture.
- Core SpecKit (v0.9.5) commands plus the bundled `agent-context` extension
  are sufficient to operate the workflow; unvetted community extensions are
  not installed (shortlist maintained separately for later approval).
- Deferred tooling: `otell` (runmat-org/otell, local-first OTLP ingest/query,
  MCP-queryable) is compatible with RunMat's `runmat-logging` `otlp` feature
  and MAY be adopted for Bundle 4 runtime-pipeline debugging (dispatch,
  acceleration, fusion) when unit tests alone do not explain a failure. Not
  installed during spec-phase work. Caveats recorded 2026-07-21: upstream
  repo currently has no licence file; use requires building a host with the
  non-default `otlp` feature, which must stay out of committed feature
  changes. All telemetry remains local; no clean-room impact.

## Dependencies and Blockers

- **B-001 — RESOLVED 2026-07-22 (second revision)**: `matlab-interface-spec`
  now has committed history; the packaged export
  `dist/runmat-export-2026-07-22` pins commit
  `b054f3ad809ceee28529a4dd7a6e28d2b6b434ef` (verified). FR-002(d) is fully
  satisfied for all 26 imported specifications; the interim content pin is
  superseded (see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`).
  Historical record of the original blocker follows.
- **BLOCKER B-001 (no citable approved export)** — REVISED 2026-07-22: the
  approval half is now resolved (20 Tier A specifications carry
  `status: approved` with a dated maintainer approval record; imported to
  `specs/imports/matlab-interface-spec/` at maintainer instruction). The
  revision half remains open: the source repository still has no commits, so
  FR-002(d) is satisfied only by an interim content pin (`SHA256SUMS`,
  manifest digest `311a11d7…daae` — see IMPORT-MANIFEST.md). Human decision
  at gate: accept the interim pin, or commit/tag the source repository and
  upgrade the pin before implementation. `median` remains draft/not exported.

## Clarifications

### Session 2026-07-21 (updated 2026-07-22)

- Q1 (approval marking): RESOLVED 2026-07-22 — approval is `status: approved`
  in the function's `interface.yaml` plus a dated approval record in the
  source repository's review docs (`docs/review/2026-07-22-tier-a-submission.md`),
  per source README "Export boundary" and source constitution §8.
- Q2 (export artefact set): RESOLVED 2026-07-22 — the export comprises
  `interface.yaml`, `behaviour.md`, `provenance.yaml`, AND
  `observations/*.json` (serialised black-box results: values, classes,
  dimensions — no documentation text), plus shared `schemas/*.json`, per the
  source README export boundary. Controlled observation files are importable;
  raw/unfiltered experiment transcripts remain prohibited (FR-004 narrowed
  accordingly).
- Q3 (release representation): Exports pin applicable releases (currently
  R2026a). How should RunMat tests represent release applicability — test
  naming/tags, doc comments, or a compatibility matrix file? → PENDING
- Q4 (approver of record): Is the repository owner the sole gate approver,
  and should gate records live in `specs/<feature>/agent-runs/` as proposed?
  → PENDING
- Q5 (first candidate): Once `matlab-interface-spec` has a committed approved
  revision, is `median` the intended first concrete built-in for the FR-010
  selection proposal, or should additional candidate exports be produced
  first? → PENDING
