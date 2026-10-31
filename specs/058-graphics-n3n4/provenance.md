# Provenance Record — 058-graphics-n3n4

| Field | `findall` | `ishghandle` | `allchild` | `pcolor` | `surfl` |
|---|---|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same | same | same |
| Approved specification | `findall` **0.2.0**, `status: approved` | `ishghandle` **0.2.0**, approved | `allchild` **0.2.0**, approved | `pcolor` **0.2.0**, approved | `surfl` **0.2.0**, approved |
| Specification revision (import pin) | **`1156c975db1f393b58252a44e04333a130211398`** (IMPORT-MANIFEST Batch 11, "N3") | same as findall | **`fc9f80485f8b5df44a2a6b1ba6aec0e8274aa6c7`** (IMPORT-MANIFEST Batch 13, "N4") | same as allchild | same as allchild |
| Review status | approved 2026-10-30 (batch N3 human-loop review) | same | approved 2026-10-30 (batch N4 human-loop review) | same | same |
| Import date | 2026-10-30 | same | same | same | same |
| Applicable release | R2026a (GLNXA64), per the imported `provenance.yaml` | same | same | same | same |

Import integrity: the 20 imported files for these five specifications pass
`sha256sum -c` against `specs/imports/matlab-interface-spec/SHA256SUMS`, and
are byte-identical to `../matlab-interface-spec/specifications/<fn>/`.

## Freshness check

2026-10-30, at start: in `../matlab-interface-spec` after
`git fetch -q origin`, `HEAD` = `origin/main` = `c3305a5` (tools only).
`git diff --stat 1156c97 origin/main -- specifications/{findall,ishghandle}`
and `git diff --stat fc9f804 origin/main --
specifications/{allchild,pcolor,surfl}` are empty; the working tree has
drafts only under `experiments/{fplot,fscanf,lsqminnorm,outedges,rat,
rectangle,reordernodes,sprandn}` — none for these functions. Repeated at
finish (see `validation.md`).

## Relied-upon claims

All present in the imported `provenance.yaml` files; every one is
`source_category: black-box-observation`, `status: confirmed`.

| claim_id | normative | used by |
|---|---|---|
| findall.signature-primary | yes | FR-058-01 |
| findall.type | yes | FR-058-02 |
| findall.hidden | yes | FR-058-03 (D-058-2) |
| findall.property | yes | FR-058-04 |
| findall.set-pattern | yes | FR-058-05 (D-058-8) |
| findall.self | yes | FR-058-01 |
| findall.no-match | yes | FR-058-06 (D-058-3) |
| findall.deleted | yes | FR-058-07 (D-058-1) |
| findall.error-not-handle | yes | FR-058-08 (D-058-1) |
| ishghandle.signature-primary | yes | FR-058-10, 11 |
| ishghandle.live | yes | FR-058-10 |
| ishghandle.type | yes | FR-058-11 |
| ishghandle.false-cases | yes | FR-058-12 |
| ishghandle.empty | yes | FR-058-13 |
| ishghandle.error-none | yes | FR-058-14 |
| allchild.signature-primary | yes | FR-058-20..24 |
| allchild.root | yes | FR-058-20 |
| allchild.figure | yes | FR-058-21 (D-058-4) |
| allchild.axes | yes | FR-058-22 |
| allchild.leaf | yes | FR-058-23 (D-058-3) |
| allchild.array | yes | FR-058-24 |
| allchild.error-deleted | yes | FR-058-25 |
| pcolor.signature-primary | yes | FR-058-30..33 |
| pcolor.output-class | yes | FR-058-30 (D-058-5) |
| pcolor.ax-arg | yes | FR-058-33 |
| pcolor.c-only | yes | FR-058-31 (D-058-6) |
| pcolor.xyc | yes | FR-058-32 |
| pcolor.current-axes | yes | FR-058-33 |
| pcolor.view | yes | FR-058-34 |
| pcolor.face | yes | FR-058-35 |
| pcolor.edgecolor | **no** | FR-058-38 (documented choice) |
| pcolor.error-size | yes | FR-058-36 |
| pcolor.error-two-args | yes | FR-058-37 |
| surfl.signature-primary | yes | FR-058-40 |
| surfl.output-class | yes | FR-058-40 (D-058-5) |
| surfl.data | yes | FR-058-41 |
| surfl.cdata | yes | FR-058-42 (D-058-7 for the numeric range) |
| surfl.view | yes | FR-058-43 |
| surfl.face | yes | FR-058-43 |
| surfl.edgecolor | **no** | FR-058-45 (documented choice) |
| surfl.error-z-2x3 | yes | FR-058-44 |
| surfl.error-size | yes | FR-058-44 |
| surfl.error-char | yes | FR-058-44 |

Backing observations (imported): `findall_observations.json` (9 cases),
`ishghandle_observations.json` (17), `allchild_observations.json` (8),
`pcolor_observations.json` (9), `surfl_observations.json` (12).

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| findall.q-warning-coverage, ishghandle.q-, allchild.q-, pcolor.q-, surfl.q-warning-coverage | no warnings raised |
| findall.q-order | RunMat tree order; tests check only the first element (FR-058-09) |
| ishghandle.q-numeric-handles | any number naming a live object is true (FR-058-15) |
| allchild.q-array-elements | each cell element is that handle's child column (FR-058-24) |
| allchild.q-order | registration order; tests compare sets (FR-058-26) |
| pcolor.q-two-arg-id | `RunMat:badsubscript` reproduced as observed; flagged for human review (FR-058-37) |
| surfl.q-min-size | fewer than 3 columns → `UnsupportedOption` (FR-058-45) |
| surfl.q-cdata | independent reflectance model (FR-058-46, D-058-7) |
| surfl.q-char-error | any single non-numeric argument → `notEnoughInputs` (FR-058-45) |
| surfl.q-error-ids | `xyzchk`/`narginchk` identifiers reproduced as observed; flagged (FR-058-44) |

## Source categories and independence

- Only the claims marked normative above are requirements. Observed
  `MATLAB:x:y` identifiers are raised as `RunMat:x:y`.
- The object tree, the deleted-handle rule, the reflectance model and all
  error identifiers for unobserved conditions are **independently derived
  documented choices**, tagged in tests (`unresolved_choice_*`,
  `deviation_*`) and not asserted as MATLAB-conformant.
- The surfl reflectance model is a textbook Lambert diffuse + Blinn-Phong
  specular model with constants chosen for RunMat; it does not attempt to
  reproduce MATLAB's values.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository.
