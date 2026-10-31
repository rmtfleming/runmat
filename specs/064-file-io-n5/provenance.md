# Provenance Record — 064-file-io-n5

| Field | `fscanf` | `urlwrite` |
|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same |
| Approved specification | `fscanf` **0.2.0**, `status: approved` | `urlwrite` **0.2.0**, `status: approved` |
| Specification revision (import pin) | **`1c91aa1`** (IMPORT-MANIFEST Batch 14, N5) | same |
| Review status | approved 2026-10-30 (source `docs/review/2026-10-30-n5-new-specs-human-loop.md`) | same |
| Observations | 16 cases | 9 cases (local `file://` URLs and an unknown protocol only) |
| Applicable release | R2026a (GLNXA64), per the imported `provenance.yaml` | same |

| Field | Value (both) |
|---|---|
| Import date | 2026-10-31 (commit `27cf58d22`) |
| Selection | maintainer, 2026-10-31, `specs/001-clean-room-matlab-builtins/selection-proposal-n5.md` (F64; G1–G3 pre-approved; `fscanf.q-mixed` reproduced as observed; `urlwrite.q-network` "Use webread transport") |
| Freshness check (start) | 2026-10-31: in `../matlab-interface-spec`, after `git fetch -q origin`, HEAD = origin/main = `1c91aa1`. No commit after the pin; `git status --short` for `specifications/{fscanf,urlwrite}` and `experiments/{fscanf,urlwrite}` is clean (no drafts). Each imported file is byte-identical to `git show 1c91aa1:specifications/<fn>/<file>` (8 files compared with `cmp`). |
| Freshness check (finish) | recorded in `validation.md` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/{fscanf,urlwrite}/{interface.yaml,behaviour.md,provenance.yaml,observations/*_observations.json}` |

## Relied-upon claims

Every id below exists in the imported `provenance.yaml` (checked with
`grep claim_id`; 10 + 7 = 17 ids, all listed, all `normative: true`,
`status: confirmed`, category `black-box-observation`).

| claim_id | used by |
|---|---|
| fscanf.signature-primary | FR-064-01, FR-064-10 |
| fscanf.c | FR-064-01 |
| fscanf.s | FR-064-02 |
| fscanf.numeric | FR-064-03 |
| fscanf.size | FR-064-04 |
| fscanf.literal-stop | FR-064-05 |
| fscanf.empty | FR-064-06 |
| fscanf.mixed | FR-064-07 (reproduction confirmed by the maintainer, `fscanf.q-mixed`) |
| fscanf.error-bad-fid | FR-064-08 |
| fscanf.error-no-format | FR-064-09 |
| urlwrite.signature-primary | FR-064-11 |
| urlwrite.success | FR-064-11 |
| urlwrite.status-failure | FR-064-12 |
| urlwrite.error-missing-one-out | FR-064-13 |
| urlwrite.error-bad-dir | FR-064-14 |
| urlwrite.error-protocol | FR-064-15 |
| urlwrite.error-one-arg | FR-064-16 |

## Unresolved questions carried

`fscanf.q-warning-coverage`, `fscanf.q-mixed`, `fscanf.q-encoding`,
`urlwrite.q-warning-coverage`, `urlwrite.q-network`, `urlwrite.q-options`.
`spec.md` ("Unresolved behaviour") records how each is handled.

## Source categories and independence

- FR-064-01..16 come from black-box observations; the identifiers are
  raised in the RunMat namespace (D-064-1).
- FR-064-20..26 are **independently derived documented choices** or
  explicit `UnsupportedOption` dispositions. Tests tag them
  `documented_choice_*`. The http/https path (FR-064-24) is a maintainer
  disposition, not observed behaviour.
- Algorithms, written for this feature or reused from RunMat:
  - fscanf: repeat the format over the rest of the file; each conversion
    is scanned by RunMat's existing `sscanf` scanner
    (`strings/core/compat.rs`), stop at the first failure; count the
    successful conversions; char result when every conversion is `%c`/`%s`;
    column-major shaping for `[m Inf]`.
  - urlwrite: parse the URL with the `url` crate, read `file://` paths with
    `runmat_filesystem`, fetch http(s) with RunMat's shared transport,
    write the bytes with `runmat_filesystem`.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
