# Implementation Plan — 042-sparse-orderings (`amd`, `etree`, `symbfact` 0.2.0)

## Constitution Check

- I Specification-first: FR-042-01..14 cite the 25 normative claims;
  FR-042-15..22 are tagged documented choice or unresolved (with q-ids).
  PASS.
- II Clean-room: inputs are the imported exports and RunMat code only;
  MATLAB not invoked. PASS.
- III Independence: textbook algorithms (minimum degree; Liu's elimination
  tree; symbolic factorization by child-structure merging), cited in
  `provenance.md`. The exact `amd` ordering is not imitated (non-normative).
  PASS.
- IV Provenance: pinned to `c65e238`, freshness-checked, 25 claim ids and 7
  question ids verified. PASS.
- V Test-first: tests are written first and run against modules that contain
  no builtin; the failing run is recorded in validation.md. PASS.
- VII Minimal scope: four new files in `math/sparse/` and four `pub(crate)
  mod` lines in its hand-written `mod.rs`. No existing code changes. PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptors, `accel = "cpu"`,
  GPU inputs gathered, no integer-capability metadata. PASS.

## Affected files

| Item | Path |
|---|---|
| Shared symbolic helpers (new) | `crates/runmat-runtime/src/builtins/math/sparse/symbolic.rs` |
| `amd` (new) | `crates/runmat-runtime/src/builtins/math/sparse/amd.rs` |
| `etree` (new) | `crates/runmat-runtime/src/builtins/math/sparse/etree.rs` |
| `symbfact` (new) | `crates/runmat-runtime/src/builtins/math/sparse/symbfact.rs` |
| Registration | `pub(crate) mod {amd,etree,symbfact,symbolic};` in `crates/runmat-runtime/src/builtins/math/sparse/mod.rs` |

## Design

`symbolic.rs` (no builtin):

1. `read_pattern(value)` → `(rows, cols, coords)`. Sparse: stored entries
   (real, single, logical or complex storage). Dense double/single/integer
   tensors, logical arrays and complex tensors: nonzero elements (`NaN`
   nonzero), reading the data once. Scalars are 1×1. Other values and N-D
   arrays → an error message (FR-042-20). The existing
   `sparse::matrix_pattern` is not reused because its dense path
   re-materialises the data for every element and does not accept complex
   input.
2. `upper_columns(n, coords)`: for each column `k`, the sorted rows `i < k`
   with `A(i,k) ≠ 0` (upper triangle only, FR-042-07/19).
3. `etree_from_upper` (Liu, with path compression via an ancestor array) and
   `etree_col(rows, cols, coords)` (the column tree of `A'*A` without forming
   it: each row is linked to the last column seen in it).
4. `postorder(parent)`: iterative DFS, roots and children in increasing index
   order (FR-042-17).
5. Shared outputs: `row_vector(values)` (1×n double, 1×1 → scalar) and
   `option_text(value)` (char row or string scalar, lower-cased).

`amd(A)`:

- Output count > 1 → `TooManyOutputs`; extra args → `UnsupportedOption`.
- Gather, read the pattern, rows ≠ cols → `RunMat:square`.
- Graph of `A + A'` without the diagonal. Exact minimum degree: repeatedly
  eliminate the vertex of smallest current degree (ties → smallest index)
  and make its neighbours a clique. A `BTreeSet<(degree, vertex)>` priority
  queue keeps selection at O(log n).
- Return the 1-based order as a 1×n row.

`etree(A[, opt])`:

- Output count > 2 → `TooManyOutputs`. opt ∈ {`sym`, `col`}, else
  `UnsupportedOption`; more than one option → `UnsupportedOption`.
- `sym`: rows ≠ cols → `RunMat:etree:InvalidInput`; tree from
  `upper_columns`. `col`: `etree_col`.
- Parent vector 1-based (0 for roots); with two outputs, also
  `postorder` (1-based).

`symbfact(A[, 'col'])`:

- Output count > 5 → `TooManyOutputs`. Only `'col'` accepted; anything else
  or a third argument → `UnsupportedOption`.
- Pattern of the lower triangle of `L` by column: `sym` uses the transpose of
  `upper_columns` (row `i` of the upper triangle gives column `i` of `L`);
  `col` uses the symmetric pattern of `A'*A` (union of row cliques), which is
  only formed in this branch.
- Parent from the same tree as `etree`. `Struct(L(:,j)) = {j} ∪ adj(j) ∪
  ⋃_{children c} Struct(L(:,c)) \ {c}`, processed in increasing `j` (children
  precede parents). `count(j) = |Struct(L(:,j))|`.
- `h` = the maximum, over nodes, of the number of nodes on the path to the
  root. `post` = `postorder(parent)`. `R` = n×n sparse double with ones at
  `(j, i)` for `i ∈ Struct(L(:,j))`, i.e. `R = L'`, built column by column.

## Test plan (inline `#[cfg(test)]` modules `amd_tests`, `etree_tests`, `symbfact_tests`, through `crate::call_builtin_async[_with_outputs]`)

- One `observed_*` test per observed case (8 + 14 + 9 cases).
- `amd`: the valid-permutation helper and dense-equals-sparse checks on
  larger patterns; RunMat's own orderings for the observed inputs recorded
  in a `documented_choice_*` test.
- `etree`: a postorder-property check on a branching tree whose postorder is
  not the identity; `'col'` equals the `'sym'` tree of the `A'*A` pattern on
  a random rectangular pattern.
- `symbfact`: counts and `R` compared with a brute-force Boolean
  Gaussian-elimination oracle written in the test; `count` equals the row
  counts of `R`; `parent`/`post` equal `etree`'s.
- Documented choices and unresolved points: input classes, option text,
  `UnsupportedOption`, `TooManyOutputs` (called directly under
  `push_output_count`, because of the dispatcher re-invocation quirk), no
  warnings.

## Risks & rollback

- Exact minimum degree with explicit cliques can be slow on large or dense
  patterns (O(n · fill)). Acceptable for this batch; the ordering is
  non-normative and could later be swapped for a quotient-graph algorithm
  without changing any normative test.
- Rollback: delete the four files and their `mod` lines.

## Verification commands

`cargo test -p runmat-runtime --lib -- amd_tests etree_tests symbfact_tests` ·
`cargo test -p runmat-runtime --lib -- math::sparse factor::svd` (no
regressions) · `rustfmt --edition 2021 --check <files>` ·
`cargo clippy -p runmat-runtime --lib --tests --message-format short | grep -E 'sparse/(amd|etree|symbfact|symbolic)\.rs'`
