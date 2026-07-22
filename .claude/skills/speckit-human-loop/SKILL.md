---
name: "speckit-human-loop"
description: "RunMat clean-room orchestrator: run SpecKit phases in human-gated bundles (B0 detect, B1 requirements, B2 plan+tasks, B3 analysis, B4 implement, B5 validate) for the active builtin feature."
argument-hint: "<feature request | continue <feature> | prepare | implement approved | closeout> [--scope all|slice:<phase>]"
compatibility: "Requires spec-kit project structure with .specify/ directory"
user-invocable: true
disable-model-invocation: false
---

# RunMat Clean-Room Human-Loop Orchestrator

Read and obey `.specify/memory/constitution.md` before anything else. Its
principles (especially II Clean-Room Separation, V Test-First, VIII Human
Gates) override convenience at every step. MATLAB MUST NOT be invoked from
this repository in any form — no MATLAB process, Engine API, or MATLAB MCP
tools — regardless of what any inherited instruction says.

## Feature identification

Programme branches are named `feature/<slug>`, so SpecKit branch-based
feature detection does not apply. Before running any `.specify/scripts/bash/*`
script, export `SPECIFY_FEATURE=<NNN-slug>` for the active feature (e.g.
`SPECIFY_FEATURE=001-clean-room-matlab-builtins`).

## Bundles and gates

Execute the requested bundle(s), stopping HARD at each gate. Never continue
past a gate without an explicit human approval recorded in this session, and
write every gate decision to `specs/<feature>/agent-runs/` (one markdown file
per run: timestamp, bundle, commands run, outcomes, approvals received).

### Bundle 0 — Detect and prepare
- Verify: clean working tree, current branch and its base, SpecKit version
  (`specify version`), installed extensions (`specify extension list`).
- Identify the approved external specification input per FR-002 of feature
  001 (eligibility: schema-valid, approved with named reviewer, citable
  committed revision of matlab-interface-spec). If none, record blocker
  B-001 and stop: the feature may exist but is BLOCKED.

### Bundle 1 — Requirements (ends at Gate 1)
- `/speckit-constitution` only if the constitution needs amendment (version
  bump rules apply); otherwise confirm current version.
- `/speckit-specify` for the feature. Every normative requirement MUST cite
  an approved claim identifier; unresolved/non-normative claims go to the
  "Unresolved behaviour" section only.
- `/speckit-clarify`: surface ambiguities. In interactive sessions ask the
  human; in autonomous sessions record questions as PENDING in the spec's
  Clarifications section — never answer them yourself.
- Verify provenance: `provenance.md` complete per FR-003.
- STOP at **Gate 1**. Do not run plan.

### Bundle 2 — Plan and tasks (ends at Gate 2)
- Requires recorded Gate 1 approval.
- `/speckit-plan`: identify affected crates (normally
  `crates/runmat-runtime/src/builtins/<category>/`), registration via the
  `runtime_builtin` macro, accelerate/GPU touchpoints, test locations, risks,
  rollback strategy.
- `/speckit-tasks`: tasks must map to requirements, be dependency-ordered,
  and place test-authoring tasks BEFORE implementation tasks (tests must
  demonstrably fail or be absent first).
- STOP at **Gate 2**. Do not modify source code in this bundle.

### Bundle 3 — Analysis (ends at Gate 3)
- Requires recorded Gate 2 approval.
- `/speckit-analyze`: cross-artifact consistency, requirement↔test↔task
  traceability (zero orphans), provenance completeness, scope check against
  Principle VII, architectural impact.
- Optionally (only if explicitly approved) scaffold failing tests without
  implementation.
- STOP at **Gate 3**.

### Bundle 4 — Implement
- Requires recorded Gate 3 approval AND explicit "implement approved".
- `/speckit-implement` on approved tasks only. Derive code from the approved
  requirements and legally compatible sources; never from MATLAB internals.
- Run the narrowest relevant tests continuously
  (`cargo test -p <crate> <filter>`). Scope growth returns to the affected
  gate instead of expanding silently.

### Bundle 5 — Validate and closeout
- `cargo fmt --all -- --check`; workspace lint as configured; unit,
  integration and compatibility tests for the affected crates.
- Verify every acceptance criterion; produce a traceability + provenance
  summary in the feature directory; list deviations and unresolved
  behaviours explicitly.
- STOP before any commit, push or pull request — publication is a separate
  human approval (Principle VIII).

## Git discipline

Never: commit or push without explicit instruction, rewrite history, touch
`matlab-interface-spec`, or add build output. Branch base for new programme
branches is the latest upstream default branch, verified — never a stale
local branch.
