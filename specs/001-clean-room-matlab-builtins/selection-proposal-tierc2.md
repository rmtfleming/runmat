# Selection Proposal (FR-010) — Batch C2 — 2026-10-29

Candidates: the eight approved specifications imported today (IMPORT-MANIFEST
Batch 6, pinned to source `104fb5f`) that neither upstream RunMat
(`main` + `RM-1064-remaining`) nor this branch implements. Branch:
`feature/clean-room-matlab-builtins` after merge `57867ce4c`.

COBRA usage (source files): `spy` 11, `incidence` 7, `exportgraphics` 7,
`findstr` 6, `validateattributes` 6, `isisomorphic` 5, `svds` 5,
`annotation` 5.

## What each approved spec confirms, and the substrate

| Function | Confirmed (normative) | Unresolved | Upstream substrate |
|---|---|---|---|
| `incidence(G)` | class `double`, sparse node×edge, 3×2 for a 3-node 2-edge graph; observed value `sparse([1;2;2;3],[1;1;2;2],[-1;1;-1;1],3,2)` (sign convention explicitly *not asserted*) | `q-shape`, `q-directed` | `graph`/`digraph` objects, edge table, `sparse` |
| `isisomorphic(G1,G2)` | 1×1 `logical`; identical graphs → `true` | `q-nonisomorphic` (no negative case observed) | graph objects, `adjacency`, `degree` |
| `findstr(s1,s2)` | class `double` row of 1-based starts; `'bc'` in `'abcabc'` → `[2 5]` | `q-order` (search shorter in longer regardless of order?), `q-deprecated` | `strfind` |
| `validateattributes(A,classes,attributes)` | silent pass for `(5,{'numeric'},{'scalar'})`; wrong type raises identifier `MATLAB:invalidType` | `q-identifiers` (other attributes and their errors) | class predicates |
| `svds(A,k)` | class `double` column of the k largest singular values, largest first: `svds(sparse([2 0;0 3]),2)` → `[3;2]` | `q-default` (`svds(A)`), `q-vectors` (`[U,S,V]`) | `svd`, `full` |
| `spy(A)` | draws into the current axes; no output | `q-markersize` | plotting (`plot`/`scatter`) |
| `exportgraphics(obj,filename)` | writes the file; no output | `q-side-effect`, `q-formats` | figure export (`print`/`saveas` path) |
| `annotation('line',x,y)` | returns an annotation graphics handle | `q-types` | no annotation object model upstream |

## Proposed grouping

| # | Feature | Function(s) | Scope | Status |
|---|---|---|---|---|
| F36 | 036-graph-structure | `incidence`, `isisomorphic` | Medium | **Decision D1** |
| F37 | 037-findstr | `findstr` | Small | **Decision D1** |
| F38 | 038-validateattributes | `validateattributes` | Medium | **Decisions D1, D2** |
| F39 | 039-svds | `svds` | Small | Ready |
| F40 | 040-spy | `spy` | Small | Ready (low value without a display) |
| F41 | 041-exportgraphics | `exportgraphics` | Small–medium | Ready (file side effect) |
| — | `annotation` | — | — | **Defer**: RunMat has no annotation object model, and the observed result is a class-specific handle |

(F35 stays reserved for the deferred `outerjoin`.)

## D1 — how strictly to treat the unresolved points this time

Batch C1 used "confirmed forms only" (D1 = A). Applied unchanged here, it
would make several builtins much less useful than their specs suggest.
Options:

- **(A) Strict, as C1.** `isisomorphic` would have to error on any
  non-identical pair, `incidence` on anything but the observed shape,
  `findstr` with a longer second argument, and `validateattributes` with any
  attribute other than `'scalar'`.
- **(B) Mixed (recommended).** Allow documented choices only where the
  observation or the function's definition fixes the answer:
  - `incidence`: reproduce the observed value's convention (−1 at an edge's
    first node, +1 at its second, column per edge in edge order) for
    undirected graphs; directed graphs give `UnsupportedOption`
    (`q-directed`).
  - `isisomorphic`: a real graph-isomorphism test (standard published
    backtracking with degree/label pruning) returning `false` for
    non-isomorphic pairs. That is what "isomorphic" means, not a guess about
    MATLAB internals.
  - `svds`: dense SVD via `svd(full(A))`, top k descending; `svds(A)` and
    `[U,S,V]` give `UnsupportedOption`.

  Stay strict where the answer is a guess:
  - `findstr`: search `s2` within `s1` only; if `s2` is longer than `s1`,
    `UnsupportedOption` (`q-order`).
  - `validateattributes`: support class matching plus the observed
    `'scalar'` attribute; every other attribute gives `UnsupportedOption`
    (`q-identifiers`).
- **(C) Permissive.** As B, plus symmetric `findstr` and a documented common
  attribute set for `validateattributes` (`positive`, `nonnegative`,
  `integer`, `real`, `finite`, `nonnan`, `nonempty`, `vector`, `row`,
  `column`, `2d`, `numel`, …), each with RunMat-chosen error identifiers.
  This covers the most COBRA call sites, but those choices are not
  observation-backed.

## D2 — `validateattributes` error identifier for a type failure

The observation records the identifier `MATLAB:invalidType`.

- **(A) Emit `MATLAB:invalidType`** (recommended). It is an observed
  public-interface fact, and code that catches errors by identifier would
  match. Upstream already emits a few `MATLAB:`-prefixed identifiers (e.g.
  `MATLAB:badsubscript`).
- **(B) Emit `RunMat:validateattributes:invalidType`**, keeping RunMat's
  namespace and treating the observed identifier as non-normative.

## Proposed order

F39 → F37 → F36 → F38 → F40 → F41 (smallest and most self-contained first).

## Decision — PENDING (maintainer, interactive gate)
## Decision — RECORDED 2026-10-29 (maintainer, interactive gate)

- [x] Scope: **all six proposed features**. F36 (incidence, isisomorphic),
  F37 findstr, F38 validateattributes, F39 svds, F40 spy,
  F41 exportgraphics. `annotation` deferred (no annotation object model).
- [x] **D1 = B (mixed).**
  - Documented choices where an observation or the definition fixes the
    answer: incidence follows the observed convention; isisomorphic is a
    real isomorphism test; svds uses a dense SVD.
  - Strict elsewhere: findstr searches s2 within s1 only (a longer s2 gives
    `UnsupportedOption`), and validateattributes supports class matching
    plus `'scalar'` only (other attributes give `UnsupportedOption`).
- [x] **D2 = B.** Type failures raise
  `RunMat:validateattributes:invalidType`; the observed `MATLAB:invalidType`
  is treated as non-normative.
- [x] Publication: batch C1 committed **locally only** before C2 starts; no
  push.

## Decision — RECORDED 2026-10-29 (maintainer, after spec widening W2)

The spec side widened all seven C2 specs to 0.3.0 (source `8614c5b`,
human-approved; imported as IMPORT-MANIFEST Batch 8).

- [x] **Implement from the updated 0.3.0 specs** (maintainer: "import the
  updated specs from the other repos and use updates specs instead").
  D1 (mixed strictness) is superseded wherever 0.3.0 resolves a point;
  questions 0.3.0 still leaves open get explicit RunMat errors or labelled
  documented choices.
- [x] **D2 = B retained.** validateattributes raises
  `RunMat:validateattributes:<suffix>` (e.g. `expectedScalar`), with the
  suffix taken from the observed identifier. The observed `MATLAB:…`
  identifiers, including the funcName middle part, are non-normative; the
  funcName and variable-name arguments may affect only the message text.
- [x] Run now, in parallel with the W1 table agents. Gates G1–G3 remain
  pre-approved; stop before any commit or push.
- `annotation` remains deferred (unchanged at 0.2.0).
