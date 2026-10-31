# Import Manifest — matlab-interface-spec exports

## Batch 14 — N5: 11 new COBRA-backlog specs — 2026-10-31

- Eleven newly approved specifications (spec 0.2.0, `status: approved`):
  `lsqminnorm`, `sprandn`, `sprank`, `outedges`, `reordernodes`, `rat`,
  `fscanf`, `urlwrite`, `fplot`, `rectangle`, `validatecolor`. Imported for
  implementation (maintainer: "then import N5 and implement it").
- Approval record: source `docs/review/2026-10-30-n5-new-specs-human-loop.md`.
  Open for review, so non-normative: the `lsqminnorm` tol rule (inferred
  from one case); `urlwrite` http(s) (only `file://` observed); `sprandn`
  values (structure only); `fplot`/`rectangle` default colours (theme);
  `rat` complex input; `fscanf` mixed numeric and `%s`; error identifiers
  that name other functions. Observed on GLNXA64/R2026a only. No earlier
  import was revised.
- Revision: committed source **`1c91aa1c9bda12a34792d49dff22727d9fcbcc20`**
  (on `origin/main`), read with `git show`; export-boundary files only.
- Byte-verified; `SHA256SUMS` regenerated (44 new lines, none changed).

---

## Batch 13 — N4: 14 new COBRA-backlog specs — 2026-10-30

- Fourteen newly approved specifications (spec 0.2.0, `status: approved`):
  `isunix`, `ismac`, `isMATLABReleaseOlderThan`, `isnat`, `pcolor`,
  `surfl`, `varfun`, `convertvars`, `addCause`, `allchild`, `allpaths`,
  `inedges`, `asManyOfPattern`, `condest`. Imported for implementation
  (maintainer: "implement N3 and N4").
- Approval record: source `docs/review/2026-10-30-n4-new-specs-human-loop.md`.
  Non-normative: pcolor/surfl default EdgeColor, asManyOfPattern
  empty-match outputs (open question), allpaths path order, the condest
  "estimate" inference. isunix/ismac/isMATLABReleaseOlderThan observed only
  on GLNXA64/R2026a. Dispatch-level `MATLAB:UndefinedFunction` is
  non-normative throughout. No earlier import was revised.
- Revision: committed source **`fc9f80485f8b5df44a2a6b1ba6aec0e8274aa6c7`**
  (on `origin/main`), read with `git show`; export-boundary files only.
- Byte-verified; `SHA256SUMS` regenerated (56 new lines, none changed).

---

## Batch 12 — cast 0.2.1 correction — 2026-10-30

- Re-import of `cast` at spec version **0.2.1** (`status: approved`),
  replacing the 0.2.0 copy from Batch 10. New normative claims
  `cast.like-complex` (a complex `p` gives a complex result of `p`'s class,
  even with zero imaginary part) and `cast.like-real-p` (a complex `A`
  'like' a real double stays complex); `q-like-complexity` is resolved
  (RunMat D-050-3). Five new observation cases (21 in total); the 16
  earlier cases are unchanged.
- Approval record: source
  `docs/review/2026-10-30-cast-0.2.1-correction-human-loop.md`.
- Revision: committed source **`c80cbcca1af8ce82ac101b151740b24ab7a270c5`**
  (on `origin/main`), read with `git show`; export-boundary files only.
- Byte-verified; `SHA256SUMS` regenerated (4 lines changed, none added).

---

## Batch 11 — N3: 14 new COBRA-backlog specs — 2026-10-30

- Fourteen newly approved specifications (spec 0.2.0, `status: approved`):
  `chdir`, `date`, `isfloat`, `orth`, `renamevars`, `rows2vars`, `sec`,
  `strread`, `struct2cell`, `strvcat`, `throw` (MException method),
  `findall`, `ishghandle`, `isomorphism`. **Imported only; not selected for
  implementation**, except that `throw` is in scope for N2 feature F49
  (`MException`), so it was imported before F49 started.
- Approval record: source `docs/review/2026-10-30-n3-new-specs-human-loop.md`.
  Non-normative (status unresolved): `orth.basis-not-unique`,
  `isomorphism.non-unique`. Dispatch-level `MATLAB:UndefinedFunction` is
  non-normative throughout. No earlier import (N1, N2, W1, W2) was revised.
- Revision: committed source **`1156c975db1f393b58252a44e04333a130211398`**
  (on `origin/main`), read with `git show`; export-boundary files only.
- Byte-verified; `SHA256SUMS` regenerated (56 new lines, none changed).

---

## Batch 10 — N2: 14 new COBRA-backlog specs — 2026-10-30

- Fourteen newly approved specifications (spec 0.2.0, `status: approved`):
  `alpha`, `keys`, `isKey`, `remove`, `hsv`, `tiledlayout`, `lastwarn`,
  `nthroot`, `textread`, `MException`, `reallog`, `addnode`, `cast`,
  `categories`. **Imported only; not selected for implementation.**
  (The backlog's lower-cased `iskey`/`mexception` are `isKey`/`MException`.)
- Approval record: source `docs/review/2026-10-30-n2-new-specs-human-loop.md`
  ("Approve all 14"). Left open for review: `cast(x,'like',complex)` gave a
  real result.
- Revision: committed source **`8f72edb119511f95884fb63f550e6c88682c327b`** (on `origin/main`), read
  with `git show`; export-boundary files only.
- Byte-verified; `SHA256SUMS` regenerated (56 new lines, none changed).

---

## Batch 9 — N1: 14 new COBRA-backlog specs — 2026-10-29

- Fourteen newly approved specifications (spec 0.2.0, `status: approved`)
  for COBRA builtins implemented by neither upstream RunMat nor this branch:
  `amd`, `etree`, `symbfact`, `rmnode`, `addedge`, `rmedge`, `laplacian`,
  `stack`, `innerjoin`, `regexptranslate`, `dec2base`, `validatestring`,
  `isprop`, `balance`. **Imported only; not yet selected for
  implementation** (an FR-010 decision for the maintainer).
- Approval record: source `docs/review/2026-10-29-n1-new-specs-human-loop.md`
  ("Approve all 14"). Five behaviours flagged as open questions (not
  requirements): rmnode ignoring unknown node IDs; innerjoin output names
  from caller variable names; exact `balance` values; exact `amd` ordering;
  `dec2base` char bases.
- Revision: committed source **`c65e2382fa2420416439f8342333af77dcaee5b3`**
  (on `origin/main`), read with `git show`; export-boundary files only.
- Byte-verified; `SHA256SUMS` regenerated (56 new lines, none changed;
  84 specifications imported in total).

---

## Batch 8 — Tier B widening W2 (batch C2 specs 0.3.0) — 2026-10-29

- Re-import of `svds`, `findstr`, `incidence`, `isisomorphic`,
  `validateattributes`, `spy`, `exportgraphics` at spec version **0.3.0**
  (`status: approved`), replacing the 0.2.0 copies from Batch 6.
  (`annotation` is unchanged at 0.2.0; still deferred.)
- Approval record: source `docs/review/2026-10-29-tier-b-w2-widening-human-loop.md`
  ("Approve all 7"). Open questions stay in each spec's
  `unresolved_questions`, including `q-warning-coverage`: an empty
  `warnings` list is not a complete statement.
- Revision: committed source **`8614c5b396851cdac1d1eb58f1cf5f7fa7f8909e`**
  ("Widen and re-approve Tier B C2 specs (W2)"), also on `origin/main`, read
  with `git show`. Only `specifications/<fn>/` export-boundary files were
  imported; source `experiments/` and the review record were NOT.
- Byte-verified; `SHA256SUMS` regenerated (28 lines for these seven specs
  replaced, no others changed).

---

## Batch 7 — Tier B widening W1 (table specs 0.3.0) — 2026-10-29

- Re-import of `addvars`, `removevars`, `movevars`, `outerjoin` at spec
  version **0.3.0** (`status: approved`), replacing the 0.2.0 copies from
  Batches 3–4. The widening resolved, by observation, every unresolved
  question carried by features 034/035: `addvars.q-position`, `q-names`;
  `removevars.q-index`, `q-multiple`; `movevars.q-after`, `q-index`;
  `outerjoin.q-keys`, `q-type` (including row order, output naming and fill
  values).
- Approval record: source `docs/review/2026-10-29-tier-b-w1-widening-human-loop.md`
  ("Approve all 4"). The three questions it flags for human review stay open
  in the specs: `removevars.q-short-mask` and `addvars.q-before-past-end`
  (non-normative), and `outerjoin.q-name-suffix-context`.
- Revision: committed source **`5d595da`** ("Widen and re-approve Tier B table
  specs (W1)"), read with `git show`. Only `specifications/<fn>/`
  export-boundary files were imported; the source's `experiments/` scripts
  and review record were NOT.
- Byte-verified; `SHA256SUMS` regenerated (16 lines for these four specs
  replaced, no others changed).

---

## Batch 6 — Tier B batches 4–6 remainder (batch C2 candidates) — 2026-10-29

- Eight approved specifications imported that neither upstream RunMat nor
  the clean-room branch implements (per the 2026-10-29 all-branch survey):
  `validateattributes`, `findstr`, `incidence`, `isisomorphic`, `svds`,
  `spy`, `exportgraphics`, `annotation`. `web`/`export` (draft stubs) and
  `median` (draft) remain excluded.
- Approval records: source `docs/review/2026-07-22-tier-b-batch4-human-loop.md`
  (approved at `29d48cd`: findstr, incidence, isisomorphic, svds),
  `…-batch5-human-loop.md` (`b52a0f7`: spy, annotation),
  `…-batch6-human-loop.md` (`b9387dc`: validateattributes, exportgraphics).
  All `status: approved` at the pinned commit.
- Revision: imported from committed source HEAD
  **`104fb5ff56756fd3a358fdc436ce011b541aff20`** ("count macro-generated
  builtins; keep clean-room inputParser") using `git show` on that commit,
  NOT the source working tree (which held uncommitted experiment edits for
  addvars/movevars/outerjoin, none of them imported).
- Export boundary: `interface.yaml`, `behaviour.md`, `provenance.yaml`,
  `observations/*.json` (+ `.gitkeep`). No experiment scripts or raw logs.
- Byte-verified against the source commit; `SHA256SUMS` regenerated over all
  70 specifications (32 new lines, none changed; `.gitkeep` excluded as
  before).

---

## Batch 5 — Tier B batch 3 — 2026-07-22

- Twelve newly approved specifications (environment / introspection / time)
  imported: `accumarray`, `eps`, `etime`, `getReport`, `license`, `methods`,
  `properties`, `seconds`, `ver`, `verLessThan`, `version`, `what`.
  (`median` remains draft — still excluded.)
- Approval record: source `docs/review/2026-07-22-tier-b-batch3-human-loop.md`
  (all 12 approved, 6 minor pre-promotion corrections).
- Revision: approved at commit **`67eaf83`**; imported at source HEAD
  **`1b019121bb747e63d69a3eb005534cb60669e5f4`** ("Clean up stale SKELETON
  labels"). The only post-approval change to these specs is a cosmetic
  `"notes"` field edit in the observation JSON (SKELETON label removed — no
  case data, values, classes, or claims changed; verified by diff). Still
  `status: approved` at HEAD. Pinned to `1b01912`.
- Byte-verified; `SHA256SUMS` regenerated over all 62 specifications.

---

## Batch 4 — Tier B batch 2 — 2026-07-22

- Twelve newly approved specifications imported, pinned to the batch-2
  approval commit **`4445294`** ("Approve Tier B batch 2 after human-loop
  review"). Verified byte-identical between `4445294` and source HEAD
  (`37087db`, which adds only unrelated draft batch-3 skeletons — not
  imported); the source working tree is mid-assembly of batch 3.
- Functions (all `status: approved` at the pinned commit; approval record:
  source `docs/review/2026-07-22-tier-b-batch2-human-loop.md`, "all 12
  approved" 2026-07-22, with four spec-side corrections — `movevars`,
  `outerjoin`, `spalloc`, `histc` — before promotion): `append`, `colamd`,
  `dissect`, `histc`, `isdir`, `isfolder`, `isobject`, `isstr`, `movevars`,
  `normalize`, `outerjoin`, `spalloc`.
- Export boundary honoured; copies verified byte-identical (`diff -r`).
  `SHA256SUMS` regenerated over all 50 specifications.

---

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
