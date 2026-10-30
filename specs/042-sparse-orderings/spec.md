# Feature Specification: `amd`, `etree`, `symbfact` — sparse symmetric orderings and symbolic analysis

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`042-sparse-orderings`; export `SPECIFY_FEATURE=042-sparse-orderings`)

**Created**: 2026-10-30

**Status**: Batch N1 (F42). Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n1.md`: "Start N1,
gates G1–G3 pre-approved").

**Input**: Approved exports `amd` 0.2.0, `etree` 0.2.0, `symbfact` 0.2.0
(see `provenance.md`).

All three builtins work on the sparsity pattern of a matrix only. They are a
tightly coupled family: `symbfact` returns the same parent vector and
postorder as `etree`, and `amd` is the ordering one applies before the
symbolic analysis.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - `amd`: a fill-reducing permutation (Priority: P1)

**Acceptance Scenarios** (normative, one per observed case):

1. `amd(sparse([4 1 0 0 1; 1 4 1 0 0; 0 1 4 1 0; 0 0 1 4 1; 1 0 0 1 4]))`
   → 1×5 double that is a permutation of 1:5 (case `perm`).
   [amd.signature-primary, amd.output-class, amd.permutation]
2. `amd` of the 4×4 arrow matrix `sparse([4 1 1 1; 1 4 0 0; 1 0 4 0; 1 0 0 4])`
   → 1×4 permutation (case `arrow`). [amd.permutation]
3. The same arrow matrix as a full matrix → the same permutation as the
   sparse call (case `dense-input`). [amd.dense-input]
4. `amd` of the 4×4 tridiagonal matrix → class double, size 1×4 (case
   `class`). [amd.output-class]
5. `amd(speye(3))` → 1×3 permutation (case `diag`). [amd.output-class,
   amd.permutation]
6. `amd(sparse(0, 0))` → 1×0 double (case `empty`); `amd(sparse(5))` → `1`
   (case `scalar`). [amd.trivial-sizes]
7. `amd(sparse(ones(2, 3)))` raises `RunMat:square` (case `err-nonsquare`).
   [amd.error-nonsquare]

### User Story 2 - `etree`: elimination tree and postorder (Priority: P1)

1. 4×4 tridiagonal → `[2 3 4 0]`; arrow → `[2 3 4 0]`; `speye(3)` →
   `[0 0 0]`; the 5×5 matrix with corner entries → `[2 3 4 5 0]` (cases
   `tridiagonal`, `arrow`, `diag`, `pentadiag`). [etree.signature-primary,
   etree.output-class, etree.parent-vector]
2. `triu` of the tridiagonal → `[2 3 4 0]`; `tril` → `[0 0 0 0]` (cases
   `upper-only`, `lower-only`). [etree.upper-triangle]
3. `etree(A, 'sym')` equals `etree(A)` (case `sym`). [etree.sym-default]
4. Full input equals sparse input (case `dense-input`). [etree.dense-input]
5. `etree(sparse([1 0; 1 1; 0 1]), 'col')` → `[2 0]` (case `col`).
   [etree.col]
6. `[p, q] = etree(...)` for the tridiagonal and arrow → `q = [1 2 3 4]`;
   for `sparse([1 0 1 0; 0 1 1 0; 1 1 1 1; 0 0 1 1])` → `p = [3 3 4 0]`,
   `q = [1 2 3 4]` (cases `postorder`, `postorder-arrow`,
   `postorder-branching`). [etree.postorder, etree.branching]
7. `etree(sparse(0, 0))` → 1×0 double (case `empty`). [etree.empty]
8. `etree(sparse([1 0; 1 1; 0 1]))` raises `RunMat:etree:InvalidInput`
   (case `err-nonsquare`). [etree.error-nonsquare]

### User Story 3 - `symbfact`: predicted Cholesky column counts (Priority: P1)

1. Tridiagonal → `[2 2 2 1]`; arrow (dense row/column first) → `[4 3 2 1]`;
   arrow with the dense row/column last → `[2 2 2 1]`; `speye(3)` →
   `[1 1 1]`; 5×5 corner matrix → `[3 3 3 2 1]` (cases `tridiagonal`,
   `arrow`, `arrow-reversed`, `diag`, `class`). [symbfact.signature-primary,
   symbfact.output-class, symbfact.counts]
2. `[count, h, parent, post, R] = symbfact(arrow)` → `[4 3 2 1]`, `4`,
   `[2 3 4 0]`, `[1 2 3 4]`, sparse upper-triangular all-ones `R` (case
   `outputs`). [symbfact.outputs, symbfact.counts]
3. Full input equals sparse input (case `dense-input`).
   [symbfact.dense-input]
4. `symbfact(sparse([1 0; 1 1; 0 1]), 'col')` → `[2 1]` (case `col`).
   [symbfact.col]
5. `symbfact(sparse([1 0; 1 1; 0 1]))` raises `RunMat:symbfact:InvalidInput`
   (case `err-nonsquare`). [symbfact.error-nonsquare]

## Requirements *(mandatory)*

### Functional Requirements (normative)

`amd`

- **FR-042-01** `p = amd(A)` MUST be accepted for a square sparse or full
  `A`; `p` MUST be a 1×n double row vector [amd.signature-primary,
  amd.output-class].
- **FR-042-02** `p` MUST be a permutation of 1:n [amd.permutation].
- **FR-042-03** A full `A` MUST give the same permutation as the equivalent
  sparse `A` [amd.dense-input].
- **FR-042-04** A 0×0 `A` MUST give a 1×0 double; a 1×1 `A` MUST give `1`
  [amd.trivial-sizes].
- **FR-042-05** A non-square `A` MUST raise `RunMat:square`
  [amd.error-nonsquare].

`etree`

- **FR-042-06** `p = etree(A)` and `p = etree(A, 'sym')` MUST be accepted and
  agree; `p` MUST be a 1×n double row vector where `p(j)` is the parent of
  node `j` in the elimination tree and `0` marks a root
  [etree.signature-primary, etree.output-class, etree.parent-vector,
  etree.sym-default, etree.branching].
- **FR-042-07** Only the upper triangle of `A` MUST be used
  [etree.upper-triangle]. Full and sparse input MUST agree
  [etree.dense-input].
- **FR-042-08** `etree(A, 'col')` MUST accept a non-square `A` and return the
  elimination tree of `A'*A` (one node per column) [etree.col].
- **FR-042-09** `[p, q] = etree(___)` MUST also return a 1×n postorder
  permutation of the tree; it MUST be `[1 2 3 4]` for the three observed
  trees [etree.postorder].
- **FR-042-10** A 0×0 `A` MUST give a 1×0 double [etree.empty]. A non-square
  `A` without `'col'` MUST raise `RunMat:etree:InvalidInput`
  [etree.error-nonsquare].

`symbfact`

- **FR-042-11** `count = symbfact(A)` MUST be accepted; `count` MUST be a 1×n
  double row vector whose `j`-th entry is the number of nonzeros (diagonal
  included) in row `j` of the upper-triangular factor pattern `R`
  [symbfact.signature-primary, symbfact.output-class, symbfact.counts].
- **FR-042-12** `[count, h, parent, post, R] = symbfact(___)` MUST return the
  tree height `h`, the elimination-tree parent vector, its postorder, and
  `R` as a sparse upper-triangular pattern of ones; for the arrow matrix
  these are `4`, `[2 3 4 0]`, `[1 2 3 4]` and the all-ones upper triangle
  [symbfact.outputs].
- **FR-042-13** Full and sparse input MUST agree [symbfact.dense-input].
- **FR-042-14** `symbfact(A, 'col')` MUST accept a non-square `A` and analyse
  `A'*A` [symbfact.col]. A non-square `A` without `'col'` MUST raise
  `RunMat:symbfact:InvalidInput` [symbfact.error-nonsquare].

### Functional Requirements (documented choices and unresolved points)

- **FR-042-15** (documented choice, amd.q-ordering; amd.specific-orderings
  is `normative: false`) The ordering is an exact minimum-degree ordering
  of the graph of `A + A'` (diagonal ignored), ties to the smallest index.
  Tests assert only FR-042-01..05. RunMat's orderings for the observed
  inputs are recorded in a `documented_choice_*` test and in
  validation.md. They are not asserted to match the reference values.
- **FR-042-16** (unresolved, amd.q-more-args) Any argument after `A` MUST
  raise `RunMat:amd:UnsupportedOption`.
- **FR-042-17** (unresolved, etree.q-postorder-general) The postorder is a
  depth-first postorder that visits roots and the children of each node in
  increasing index order. Tests check the postorder property on a tree whose
  postorder differs from the identity. The specific sibling order is a
  documented choice.
- **FR-042-18** (unresolved, symbfact.q-more-options) For `symbfact`, any
  option other than `'col'` (including `'sym'`, which is not observed for
  `symbfact`) and any third argument MUST raise
  `RunMat:symbfact:UnsupportedOption`. For `etree`, an option other than
  `'sym'`/`'col'` or a third argument raises `RunMat:etree:UnsupportedOption`
  (documented choice). Option text may be a char row or a string scalar and
  is matched case-insensitively (documented choice).
- **FR-042-19** (documented choice) `symbfact` uses only the upper triangle
  of a square `A`, consistent with `etree` (FR-042-07), so its `parent` and
  `post` outputs equal those of `etree`. `h` is the number of nodes on the
  longest leaf-to-root path (`1` for a diagonal matrix, `0` for a 0×0
  matrix). `R` is an n×n sparse double holding `1` at each predicted
  nonzero.
- **FR-042-20** (documented choice) Accepted inputs: dense or sparse double,
  single, logical and complex matrices, and numeric/logical scalars. The
  pattern is the set of stored nonzeros (sparse) or of nonzero elements
  (dense; `NaN` counts as nonzero). Char, cell, struct, string and other
  values, and arrays with more than two dimensions, raise
  `RunMat:<fn>:InvalidInput`. GPU inputs are gathered to the host.
- **FR-042-21** (documented choice) Too many outputs (`amd` > 1, `etree` > 2,
  `symbfact` > 5) raise `RunMat:<fn>:TooManyOutputs`. Fewer outputs return
  the leading ones.
- **FR-042-22** (unresolved, amd/etree/symbfact.q-warning-coverage) No code
  path emits a warning.

## Decisions

- **Error identifier namespace.** As in previous batches, observed `MATLAB:`
  identifiers are raised in RunMat's namespace with the same suffix:
  `MATLAB:square` → `RunMat:square`, `MATLAB:etree:InvalidInput` →
  `RunMat:etree:InvalidInput`, `MATLAB:symbfact:InvalidInput` →
  `RunMat:symbfact:InvalidInput`.
- **Exact `amd` ordering is non-normative** (maintainer decision in
  `selection-proposal-n1.md`).

## Success Criteria *(mandatory)*

- **SC-042-1**: every observed case (8 `amd`, 14 `etree`, 9 `symbfact`) has an
  `observed_*` test that checks class, size and the normative value or
  error.
- **SC-042-2**: the three builtins are registered through
  `#[runtime_builtin]`, and
  `cargo test -p runmat-runtime --lib -- amd_tests etree_tests symbfact_tests`
  passes.
- **SC-042-3**: every FR maps to at least one test (validation.md).

## Unresolved behaviour (returned to the specification side)

| id | disposition |
|---|---|
| amd.q-warning-coverage | no warnings emitted (FR-042-22) |
| amd.q-more-args | `RunMat:amd:UnsupportedOption` (FR-042-16) |
| amd.q-ordering | ordering non-normative; minimum-degree choice documented (FR-042-15) |
| etree.q-warning-coverage | no warnings emitted (FR-042-22) |
| etree.q-postorder-general | documented choice: increasing-index DFS postorder (FR-042-17) |
| symbfact.q-warning-coverage | no warnings emitted (FR-042-22) |
| symbfact.q-more-options | `RunMat:symbfact:UnsupportedOption` (FR-042-18) |

## Assumptions

- The observed inputs are reconstructed from the `call` fields of the
  imported observation files.
