<!--
Sync Impact Report
==================
Version change: (template) → 1.0.0
Modified principles: n/a (initial adoption)
Added sections:
  - Core Principles I–X (specification-first, clean-room separation,
    implementation independence, provenance, test-first, traceability,
    minimal scope, human gates, RunMat compatibility, licensing)
  - Clean-Room Boundary and Repository Roles
  - Development Workflow and Human Gates
  - Governance
Removed sections: none
Templates requiring updates:
  - .specify/templates/plan-template.md ✅ compatible (Constitution Check
    section is generic; gates below map onto it)
  - .specify/templates/spec-template.md ✅ compatible (mandatory sections
    unchanged; provenance metadata is additive per-feature)
  - .specify/templates/tasks-template.md ✅ compatible (test-first task
    ordering is already the template default)
Follow-up TODOs:
  - TODO(APPROVED_EXPORT_REVISION): matlab-interface-spec has no committed
    revision yet; provenance pinning requires a commit/tag in that repository.
-->

# RunMat Clean-Room MATLAB Builtins Constitution

Scope: this constitution governs the programme of implementing additional
MATLAB-compatible built-in functions in RunMat from independently produced
interface specifications. It binds every SpecKit feature under `specs/` that
belongs to this programme, and every agent or human contributor working on
such a feature.

## Clean-Room Boundary and Repository Roles

Two repositories participate in a clean-room workflow:

- `matlab-interface-spec` (specification side): produces reviewed,
  implementation-neutral interface specifications and controlled black-box
  observations. MATLAB may be invoked only there.
- `runmat` (implementation side, this repository): consumes approved
  specification exports and produces an independent Rust implementation.

Material MUST flow in one direction only: approved specification artefacts
from `matlab-interface-spec` into RunMat feature specifications. Nothing in
this repository may be copied back into `matlab-interface-spec` by automated
tooling, and raw observation logs, experiment transcripts and protected
documentation MUST NOT be copied into this repository.

## Core Principles

### I. Specification-First Development

Every built-in implemented under this programme MUST have a SpecKit feature
specification under `specs/` before any implementation work begins.
Implementation MUST NOT begin from a function name, an informal request, or
general knowledge of MATLAB alone. Each normative behavioural requirement in
a feature specification MUST originate in an approved interface specification
exported from `matlab-interface-spec`. Behaviour that has no approved source
MUST be recorded as unresolved, not invented.

### II. Clean-Room Separation (NON-NEGOTIABLE)

RunMat MUST receive only approved, independently worded specification
artefacts. The following MUST NOT enter this repository in any form: MathWorks
source code; copied or paraphrased-with-copying MathWorks documentation prose;
MathWorks documentation examples; raw extraction material; unreviewed
experiment logs. MATLAB MUST NOT be invoked from this repository: no MATLAB
process, MATLAB MCP tool, or MATLAB Engine call may be used in any RunMat
feature workflow, test, build step, or agent action. Contributors who have
viewed protected MathWorks material outside the approved specification chain
MUST surface that fact rather than contribute tainted content.

### III. Implementation Independence

Implementations MUST be derived from the approved behavioural requirements,
RunMat's own architecture, and legally compatible technical sources (e.g.
textbooks, published algorithms, permissively licensed code whose licence is
verified). Contributors MUST NOT infer, reconstruct or imitate MATLAB internal
algorithms; equivalent observable behaviour does not require an identical
internal algorithm, and algorithm choice is free wherever the approved
specification does not constrain observable behaviour.

### IV. Provenance

Every feature MUST record, in a `provenance.md` (or equivalent structured
file) inside its feature directory: the source repository; the approved
specification identifier; the specification version or commit; the review
status; and the import date. Requirements MUST distinguish three source
categories — black-box observations, public-interface facts, and
independently derived inferences — and inferences MUST NOT be presented as
observations. Behaviour that the approved specification leaves unresolved
MUST remain explicitly marked unresolved in the feature specification, its
tests, and its documentation. If the specification repository has no
committed, citable revision, the feature is blocked at Gate 1 and MUST NOT
proceed to implementation.

### V. Test-First Development (NON-NEGOTIABLE)

Acceptance criteria MUST be defined before implementation. Tests MUST cover
returned values, result types/classes, result shapes, error conditions,
warning conditions, and the edge cases named in the approved specification.
Where practical, tests demonstrating the absent or failing behaviour MUST
exist before the implementation step. Release-dependent or platform-dependent
behaviour MUST be represented explicitly (e.g. tagged by applicable release)
rather than silently generalised. No test result may be fabricated from
memory: every reported test outcome MUST come from an actual test run in this
repository.

### VI. Traceability

Each normative requirement MUST map to one or more acceptance criteria and
tests; each implementation task MUST map back to at least one requirement;
and each feature's validation report MUST identify the concrete evidence
(test names, command output) supporting completion. Orphan tests SHOULD be
flagged during analysis. A requirement with no covering test, or a task with
no requirement, blocks the relevant gate.

### VII. Minimal Scope

Each feature MUST implement one coherent built-in or one tightly coupled
family of built-ins. Unrelated refactoring MUST NOT be bundled into a feature.
Pull requests SHOULD be small and reviewable. Scope changes discovered during
implementation MUST return to the specification and re-pass the affected
gates rather than being absorbed silently.

### VIII. Human Gates

Human approval is REQUIRED at three points: Gate 1 after requirements
(specification + clarification + provenance check); Gate 2 after planning and
task decomposition; Gate 3 after cross-artifact analysis and before
implementation. Implementation approval and publication approval (commit,
push, pull request) are separate: approval to implement MUST NOT be treated
as approval to publish. Legal questions, provenance gaps and behavioural
ambiguity MUST be surfaced to the human reviewer rather than silently
resolved. Agents MUST stop at each gate and wait.

### IX. RunMat Compatibility

Implementations MUST follow RunMat's existing architecture and conventions:
builtins live under `crates/runmat-runtime/src/builtins/` in the appropriate
category, are registered through the existing `runtime_builtin` macro
machinery in `runmat-macros`/`runmat-builtins`, and use RunMat's runtime
`Value` representation. CPU/GPU split, accelerator dispatch and value
semantics MUST be considered where the builtin touches them. Code MUST pass
`cargo fmt` and existing lint/test conventions. New architecture MUST NOT be
introduced solely to imitate MATLAB internals. Builtin documentation MUST
follow `docs/builtins/authoring.md` where applicable.

### X. Licensing

MathWorks source code, decompiled artefacts, and copied expressive
documentation MUST NOT be used. GPL-derived or otherwise licence-incompatible
implementation code MUST NOT be transferred into RunMat (RunMat's licence and
NOTICE files govern compatibility). The licence and provenance of every
external implementation source MUST be reviewed and recorded in the feature's
provenance file. Completion reports MUST record that independent legal review
MAY still be required; this constitution does not substitute for legal advice.

## Development Workflow and Human Gates

The lifecycle for every builtin feature is:

approved interface specification → SpecKit feature specification →
clarification → implementation plan → task decomposition → cross-artifact
analysis → human approval → implementation → tests and validation → review
and pull request.

Phases and gates (executed with core SpecKit commands):

- B0 Detect and prepare: verify repository cleanliness, branch base,
  SpecKit/extension versions, and the approved external specification input.
- B1 Requirements: `/speckit-specify`, then `/speckit-clarify`; update the
  spec; verify clean-room provenance. STOP at Gate 1.
- B2 Plan and tasks: `/speckit-plan`, then `/speckit-tasks`; identify
  affected crates, registration points, tests, risks, rollback. STOP at
  Gate 2.
- B3 Analysis: `/speckit-analyze`; check requirement-to-test traceability,
  provenance completeness, scope and architectural impact. STOP at Gate 3.
- B4 Implementation: `/speckit-implement` on approved tasks only; keep tests
  running narrowly; no scope broadening without returning to the affected
  gate.
- B5 Validation: `cargo fmt --all -- --check`, `cargo clippy` where
  configured, unit/integration/compatibility tests; produce traceability and
  provenance summary. STOP before commit/push/PR unless separately approved.

Because programme branches are named `feature/<slug>` rather than SpecKit's
numbered convention, `SPECIFY_FEATURE=<NNN-slug>` MUST be exported (or the
equivalent explicit feature argument used) when running SpecKit scripts, so
feature detection does not depend on the branch name.

## Governance

This constitution supersedes ad-hoc practice for all features in this
programme. Amendments require: a documented change, a semantic version bump
(MAJOR for principle removals/redefinitions, MINOR for additions or material
expansions, PATCH for clarifications), human approval, and propagation to
dependent templates. Every plan's Constitution Check section MUST verify
compliance with Principles I–X, and every gate review MUST confirm no
principle has been violated since the previous gate. Complexity beyond the
approved specification MUST be justified in the plan or rejected.

**Version**: 1.0.0 | **Ratified**: 2026-07-21 | **Last Amended**: 2026-07-21
