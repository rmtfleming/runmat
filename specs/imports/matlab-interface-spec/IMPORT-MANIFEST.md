# Import Manifest — matlab-interface-spec Tier A export (batch 1)

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
