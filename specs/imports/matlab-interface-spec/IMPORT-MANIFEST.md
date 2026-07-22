# Import Manifest — matlab-interface-spec exports

## Batch 3 — Tier B batch 1 — 2026-07-22

- Twelve newly approved specifications imported directly from the source
  repository at **commit `e46587ae1b78237e0f43cd55f678de01a28495a4`**
  ("Assemble Tier B batch 2 skeletons"), verified clean at HEAD for all
  twelve (the source working tree also holds *draft* Tier B batch-2 files,
  which were NOT imported).
- Functions (all `status: approved`, verified in each `interface.yaml`;
  approval record: source `docs/review/2026-07-22-tier-b-batch1-human-loop.md`,
  "all 12 approved" 2026-07-22, with two spec-side observation fixes to
  `nonzeros` and `dec2bin` before promotion): `addvars`, `computer`,
  `dec2bin`, `iscellstr`, `iscolumn`, `isrow`, `issparse`, `istable`,
  `nonzeros`, `psi`, `removevars`, `strncmpi`.
- No packaged `dist/` export exists for Tier B; import is directly from the
  committed `specifications/` tree at the pinned commit (export boundary
  honoured: interface.yaml, behaviour.md, provenance.yaml, observations/*.json).
- Copies verified byte-identical to source (`diff -r`). `SHA256SUMS`
  regenerated over all 38 specifications.
- Note: source `provenance.yaml` author for this batch is "clean-room agent
  (Claude Opus 4.8)" (Tier A batch was Claude Fable 5) — recorded for
  traceability; approval is by the same repository maintainer.

---

# Import Manifest — matlab-interface-spec Tier A exports

## Batch 2 — 2026-07-22 (supersedes the interim pin below)

- Source: `dist/runmat-export-2026-07-22/` in the source repository —
  packaged export with `EXPORT_MANIFEST.json` (copied alongside this file).
- **Revision pin (authoritative): commit
  `b054f3ad809ceee28529a4dd7a6e28d2b6b434ef`** of `matlab-interface-spec`
  ("Approve remaining six specs after human-loop review"), verified to exist
  in the source repository. **Blocker B-001 is RESOLVED**; the interim
  content pin of batch 1 is superseded. FR-002(d) is now fully satisfied
  for all 26 imported specifications.
- Added six newly approved specifications (all verified `status: approved`
  in their `interface.yaml`): `display`, `inputParser`, `saveas`, `system`,
  `urlread`, `writetable`. The original 20 were verified byte-identical
  between batch 1 and the packaged export (no drift).
- `SHA256SUMS` regenerated over all 26 (109 content files); drift detection
  unchanged: `sha256sum -c SHA256SUMS`.
- No features exist yet for the six new functions; they await selection
  proposals under FR-010. Note: all six are side-effect functions (process
  execution, network, file writing, display) — their features will need
  explicit sandbox/isolation planning at the gate.

---

# Batch 1 — original import record (2026-07-22, interim pin — superseded)

**Import date**: 2026-07-22
**Imported by**: Claude Fable 5 (Claude Code), at maintainer instruction
**Source repository**: `matlab-interface-spec` (local path
`/home/rfleming/drive/sbgCloud/code/matlab-interface-spec`; no remote recorded)

## Approval record

- Review submission: `docs/review/2026-07-22-tier-a-submission.md` in the
  source repository — **APPROVED 2026-07-22** by the repository maintainer.
- Per-specification `status: approved` verified in each `interface.yaml` at
  import time (all 20).
- Export boundary honoured (source README "Export boundary" + constitution
  §1.4/§8): only `interface.yaml`, `behaviour.md`, `provenance.yaml`,
  `observations/*.json` per function, plus shared `schemas/*.json`.
  Draft/pending specifications (`inputParser`, `writetable`, `saveas`,
  `system`, `urlread`, `display`, `median`) were NOT imported.

## Revision pin

- **Source commit**: UNAVAILABLE — the source repository has no commits
  (unborn `main`). FR-002(d) of feature 001 is therefore satisfied only by
  an interim **content pin**: `SHA256SUMS` in this directory (83 files;
  manifest digest
  `311a11d7b1854d3337dbb9f4d9953fb7c1957d7ec3cab8facf3062e34a05daae`).
- Drift detection: re-run `sha256sum -c SHA256SUMS` against this directory;
  compare against the source before any re-import.
- ACTION REQUIRED (recorded at Gate review): once the source repository has a
  committed/tagged revision containing these artefacts, upgrade this pin to
  that commit hash.

## Imported specifications (all version 0.2.0, MATLAB R2026a, GLNXA64)

array2table, bounds, cell2table, datestr, fields, fileparts, full, int2str,
iscell, isfile, isstruct, matches, num2cell, spdiags, speye, str2num,
strmatch, strtok, table2array, table2cell

## Usage rules (constitution Principles I–IV)

- These artefacts are the ONLY source of normative MATLAB behaviour for
  builtin features. Claims marked `independently-derived-inference` or
  `unresolved` are NON-NORMATIVE.
- Each builtin feature cites claims by `claim_id` from the function's
  `provenance.yaml`.
- Files under this directory are read-only reference material; edits happen
  only on the specification side followed by a fresh export and manifest
  update.
