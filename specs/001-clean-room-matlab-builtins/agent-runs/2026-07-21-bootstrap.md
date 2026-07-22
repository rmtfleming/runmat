# Agent Run Ledger — 2026-07-21 — Bootstrap (B0 + B1)

**Agent**: Claude Fable 5 (Claude Code), autonomous session
**Branch**: `feature/clean-room-matlab-builtins` @ 97f3a0a01a8949b993cef001b4064508a406ed91 (= origin/main, verified upstream default)
**SpecKit**: specify CLI 0.9.5; bundled extension `agent-context` v1.0.0

## Bundle 0 — Detect and prepare
- Working tree clean at start; branch created from `origin/main` after fetch.
- `origin` = https://github.com/runmat-org/runmat.git (canonical upstream,
  read-only for this account: push=false). No fork configured. No pushes made.
- UPDATE 2026-07-21 (post-report): user created fork rmtfleming/runmat and
  approved remote reconfiguration. Remotes now conventional:
  `upstream` = https://github.com/runmat-org/runmat.git (canonical),
  `origin` = git@github.com:rmtfleming/runmat.git (writable fork, verified
  via SSH fetch; fork main = 97f3a0a01, in sync with upstream). Branch
  tracking updated to `upstream/main`. Still no pushes made.
- Approved external specification input: NONE eligible — matlab-interface-spec
  has no commits; candidate `specifications/median/` lacks a citable revision.
  Recorded as blocker B-001.

## Bundle 1 — Requirements
- Constitution created: `.specify/memory/constitution.md` v1.0.0.
- Feature specification created: `spec.md` (governance feature; no built-in
  behaviour defined).
- Clarification run in autonomous mode: Q1–Q5 recorded PENDING in spec.md.
- Provenance record created: `provenance.md` (nothing imported; B-001).
- Checklist: `checklists/requirements.md` — two items open pending Gate 1.

## Baseline verification (toolchain rust 1.90.0 per rust-toolchain.toml)
- `cargo fmt --all -- --check` — clean (exit 0).
- `cargo check -p runmat-builtins` — Finished in 19.72s, no errors.
- `RUST_TEST_THREADS=1 cargo test -p runmat-builtins` — 5 passed, 0 failed.
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime-integration-tests --test dispatcher`
  — 3 passed, 0 failed (builtin dispatch incl. GPU-argument gather paths).
- Repo-documented full baseline (docs/development/testing.md) is broader
  (workspace clippy/check/test); narrow subset run deliberately per task scope.

## Gate 1 — PENDING HUMAN APPROVAL
- Awaiting: answers to Q1–Q5; approval of constitution v1.0.0; decision on
  extension shortlist; resolution path for B-001 (commit + approval marking
  in matlab-interface-spec); fork/remote decision for future PRs.
- No plan, tasks, analysis or implementation performed. No commit, no push,
  no PR. matlab-interface-spec not modified.
