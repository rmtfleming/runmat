# Validation Report — 048-containers-map-conformance (specs 0.2.0) — 2026-10-30

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the cargo runner wrapper. The target directory was shared with other
agents. MATLAB was not invoked.

## Freshness check (T0, T8)

In `../matlab-interface-spec`, after `git fetch -q origin` (rc 0):
`HEAD` = `origin/main` = `1156c97` (later than the pin `8f72edb`);
`git log 8f72edb..HEAD` and `8f72edb..origin/main` over
`specifications/{keys,isKey,remove}` were empty and `git status --short`
for them was clean — at the start of the work and again at the end
(2026-10-30 08:52 UTC). The imported 0.2.0 copies are current.

## Test-first evidence (T2–T3)

`conformance_tests.rs` was written and registered before any change to
upstream code. `cargo test -p runmat-runtime --lib -- map_conformance_tests`
against unchanged upstream: **11 passed, 8 failed, 0 ignored**. Failing:

| test | upstream behaviour observed |
|---|---|
| `observed_iskey_cell_query_same_shape` | `[true false true true]` column-major vs expected `[true true false true]` (row-major cell payload written into a column-major logical array) |
| `observed_iskey_lenient_type_mismatch_and_empty_map` | char key on numeric map raised a KeyType error |
| `unresolved_choice_iskey_mismatch_inside_cell_is_false` | same KeyType error |
| `observed_remove_absent_key_warns_and_completes` | raised `RunMat:containers.Map:MissingKey` |
| `observed_remove_partly_absent_removes_present` | raised MissingKey |
| `observed_remove_type_mismatch_warns` | raised a KeyType error |
| `unresolved_choice_remove_warning_respects_error_mode` | `RunMat:containers.Map:Internal` (see D-048-1) |
| `observed_keys_method_syntax_forms` | `M.keys` reached `containers.Map.subsref`, which has no member `keys` (that run, in parallel, reported "internal storage not found"; single-threaded it is the unknown-property path) |

## Upstream behaviour changes (all in `containers.map.rs`)

1. **`isKey`**: a text key on a numeric-keyed map (or a numeric/logical key
   on a char map) now returns `false` instead of raising the KeyType error
   (new helper `key_category_mismatch`). Numeric-class mismatches (e.g. a
   `double` key on a `uint64` map) still raise the KeyType error, so the
   upstream test `integer_key_methods_reject_class_mismatch_and_binary64_aliases`
   is unchanged and passes. [FR-048-08, U-048-3]
2. **`isKey` cell queries**: `collect_key_spec` now walks a cell in
   column-major order, so element (i,j) of the logical result answers cell
   element (i,j). This also changes the order in which `remove` visits a
   cell of keys (no observable effect). [FR-048-06]
3. **`remove`**: an absent key, or a text key on a numeric map (and vice
   versa), no longer raises `RunMat:containers.Map:MissingKey` / the KeyType
   error. All present keys are removed, then one warning
   `RunMat:Containers:Map:NoKeyToRemove` ("containers.Map: a key to remove is
   not in the map") is issued through `crate::warnings::emit`.
   Numeric-class mismatches still raise the KeyType error. [FR-048-12,
   U-048-2, U-048-3]
4. **`M.keys`**: `containers.Map.subsref` now answers the member `keys`
   (exact spelling) with `keys(M)`. [FR-048-04]
5. **Upstream test updated**: `remove_missing_key_returns_error` (asserted
   MissingKey) was replaced by `remove_missing_key_warns_and_keeps_map`
   (asserts the warning identifier and that the map keeps its key).

`values(M, keySet)` and `M(key)` with a missing key still raise
`RunMat:containers.Map:MissingKey` (unchanged; outside these specs).

## Commands and results (T7)

- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib --
  map_conformance_tests exceptions_warnings_tests`: **41 passed, 0 failed,
  2 ignored** (18 F48 tests pass, 1 ignored:
  `deviation_remove_warning_error_mode_through_dispatcher`; the rest are
  F49). The same filter with default parallel threads: **41 passed, 0
  failed, 2 ignored**.
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib --
  builtins::containers builtins::diagnostics warnings:: object::`: **224
  passed, 1 failed, 2 ignored**. The failure,
  `containers_map::tests::containers_map_descriptor_includes_constructor_and_method_signatures`,
  is **pre-existing and unrelated**: it asserts the signature label
  `value = containers.Map.subsref(M, kind, payload)` while upstream's
  descriptor says `value = containers.Map.subsref(M, S)`; this feature's
  diff does not touch any descriptor or signature label.
- `rustfmt --edition 2021 --check` on `conformance_tests.rs`,
  `containers.map.rs`, `map/mod.rs`: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  first run reported one finding in this feature
  (`clippy::cloned_ref_to_slice_refs`, `conformance_tests.rs:303`), fixed
  with `std::slice::from_ref`; re-run result in the final section below.

## Traceability (claim → FR → test → result) (T8)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| keys.signature-primary, keys.output-class | FR-048-01 | `observed_keys_char_sorted_row_cell`, `observed_keys_numeric_sorted`, `observed_keys_empty_map_is_1x0` | PASS |
| keys.sorted | FR-048-02 | `observed_keys_char_sorted_row_cell`, `observed_keys_numeric_sorted`, `observed_keys_case_order_uppercase_first` | PASS |
| keys.aligned-with-values | FR-048-03 | `observed_values_aligned_with_keys` | PASS |
| keys.method-syntax | FR-048-04 | `observed_keys_method_syntax_forms` (`M.keys`, `M.keys()`, qualified name) | PASS (after fix 4) |
| isKey.signature-primary, isKey.output-class, isKey.membership | FR-048-05 | `observed_iskey_present_absent_case` | PASS |
| isKey.cell-query | FR-048-06 | `observed_iskey_cell_query_same_shape`, `observed_iskey_numeric_and_numeric_cell` | PASS (after fix 2) |
| isKey.numeric | FR-048-07 | `observed_iskey_numeric_and_numeric_cell` | PASS |
| isKey.lenient | FR-048-08 | `observed_iskey_lenient_type_mismatch_and_empty_map` | PASS (after fix 1) |
| isKey.string-key | FR-048-09 | `observed_iskey_string_scalar_key` | PASS |
| remove.signature-primary, remove.removes | FR-048-10 | `observed_remove_one_key_handle_semantics`, `observed_remove_several_numeric_all` | PASS |
| remove.handle, remove.returns-map | FR-048-11 | same two tests, `observed_remove_absent_key_warns_and_completes` | PASS |
| remove.warning-absent, remove.present-removed-with-absent | FR-048-12 | `observed_remove_absent_key_warns_and_completes`, `observed_remove_partly_absent_removes_present`, `observed_remove_type_mismatch_warns`; upstream `remove_missing_key_warns_and_keeps_map` | PASS (after fix 3) |
| keys.non-map (non-normative), U-048-1 | FR-048-13 | `unresolved_choice_keys_non_map_errors` | PASS |
| U-048-2 | FR-048-13 | `observed_remove_*` (one warning per call); error mode: `deviation_remove_warning_error_mode_through_dispatcher` | PASS / ignored (D-048-1) |
| U-048-3 | FR-048-13 | `unresolved_choice_iskey_mismatch_inside_cell_is_false`; upstream `integer_key_methods_reject_class_mismatch_and_binary64_aliases` | PASS |
| *.q-warning-coverage | FR-048-14 | no warning path besides FR-048-12 (not tested) | not tested |

Every FR maps to at least one test; every test cites a claim or a documented
choice.

## Deviations

- **D-048-1** Through the by-name dispatcher an error from
  `containers.Map.remove` is followed by a second invocation of the same
  method (dispatcher implementation loop; confirmed with a temporary
  backtrace print, since removed), which reports
  `RunMat:containers.Map:Internal` "internal storage not found" and masks the
  real error. Reproduced deterministically single-threaded for the
  `warning('error', id)` case; it also masked upstream's MissingKey error
  in the pre-fix run. Upstream dispatch issue; not fixed (no refactor of
  upstream subsystems). Ignored test kept.
- Parallel-run note: in one parallel run before the fixes,
  `observed_keys_method_syntax_forms` also reported "internal storage not
  found"; final runs (parallel and single-threaded) pass.

## Notes

- Integer-capability metadata was not touched.
- `containers/map/mod.rs` is hand-written (not generated); it gained a
  `#[cfg(test)] #[path = "conformance_tests.rs"] mod map_conformance_tests;`.
- No MathWorks material was consulted; MATLAB was not invoked.

## Final checks

- `cargo clippy -p runmat-runtime --lib --tests --message-format short`
  after the fix: **no findings anywhere in the crate** (`Finished dev
  profile`), lib and tests targets.
- Re-run after the clippy fix: `RUST_TEST_THREADS=1 cargo test -p
  runmat-runtime --lib -- map_conformance_tests exceptions_warnings_tests`
  → **41 passed, 0 failed, 2 ignored**.
- Final freshness check (2026-10-30 08:52 UTC): clean, as above.
