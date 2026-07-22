<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan
<!-- SPECKIT END -->

# Clean-Room MATLAB Builtins Programme

This repository is the implementation half of a two-repository clean-room
workflow. Binding rules live in `.specify/memory/constitution.md` — read it
before any work on a `specs/` feature. Non-negotiables:

- MATLAB MUST NOT be invoked from this repository (no process, Engine, or
  MATLAB MCP tools). Behavioural requirements come only from approved exports
  of the `matlab-interface-spec` repository.
- No MathWorks source, documentation prose, documentation examples, or raw
  observation logs may enter this repository.
- No built-in implementation before SpecKit Gates 1–3 are human-approved.
  Orchestrate with `/speckit-human-loop` (see
  `.claude/skills/speckit-human-loop/SKILL.md`).
- Programme branches are `feature/<slug>`; export
  `SPECIFY_FEATURE=<NNN-slug>` (e.g. `001-clean-room-matlab-builtins`)
  before running `.specify/scripts/bash/*` scripts, since feature detection
  cannot use these branch names.
- Do not commit, push, or open PRs without explicit human instruction.
