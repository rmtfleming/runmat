# Implementation Plan — 030-accumarray

## Constitution Check

- I Specification-first: FR-030-01 cites the three approved claims; all other
  FRs are explicitly tagged documented-choice/unresolved — PASS
- II Clean-room: sources = imported export + RunMat code only; MATLAB not
  invoked — PASS
- III Independence: scatter-add derived from RunMat's `Tensor`/`Value` model,
  not from any MATLAB internal algorithm — PASS
- IV Provenance: `provenance.md` complete; revision pinned to source commit
  `1b01912` — PASS
- V Test-first: acceptance criteria and test tiers defined before the builtin
  body; observed case asserted against the export — PASS
- VII Minimal scope: one builtin, one file, one `mod` line; no refactoring — PASS
- IX Compatibility: lives under `array/creation/`, registered via the
  `#[runtime_builtin]` inventory macro, uses `Value`/`Tensor`, GPU gather via
  the shared helper; passes `cargo fmt`/`clippy` — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `accumarray` builtin | `crates/runmat-runtime/src/builtins/array/creation/accumarray.rs` (new) |
| Registration | `pub(crate) mod accumarray;` added to `array/creation/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Placement rationale: `accumarray` constructs an output array from
subscripts + values — the direct analogue of `sparse(i,j,v)`, which already
lives in `array/creation`. Exemplars followed:
`array/creation/sparse.rs` (subs+vals construction, descriptor/error shape),
`array/indexing/sub2ind.rs` (subscript validation, GPU gather, host compute),
`math/reduction/bounds.rs` (descriptor + inline test-tier style).

## Design decisions

- Signature `accumarray_builtin(subs: Value, vals: Value, rest: Vec<Value>)`;
  `rest[0]=sz`, `rest[1]=fun`, `rest[2]=fillval`, `rest[3]=issparse`.
- Gather GPU-resident inputs with `gpu_helpers::gather_value_async`, then
  `tensor::value_into_tensor_for` (coerces logical/int → double). Host-only
  scatter-add; GPU spec `op_kind = Custom("scatter")`,
  `residency = GatherImmediately`.
- Subscript shape: `rows = shape[0]`, `cols = numel / rows`. `cols > 1`
  (row vector or N-by-K matrix) → multi-column form → reject
  (`MultiColumnUnsupported`). This matches MATLAB's N-by-K interpretation.
- Validate each subscript as finite, integer (`fract == 0`), `>= 1`.
- `vals`: scalar → broadcast; else length must equal N.
- `sz` (single-subscript `[m 1]`): first element is length `m`; trailing
  elements must be `1`; `m >= max(subs)`; `[]` → `max(subs)`.
- `fillval`: scalar; applied only where `touched[k] == false`.
- `fun`: only `[]` accepted; a function handle or any non-empty value →
  `CustomFunctionUnsupported`.
- `issparse` (a 4th trailing option) → `SparseOutputUnsupported`.
- Output: `Tensor` shape `[m,1]`; collapse to `Value::Num` when `m == 1`
  (RunMat 1×1 convention); `m == 0` → empty `[0,1]`.
- `type_resolver` returns `column_vector_type()` (double column, length
  unknown at type time).

## Test plan (inline `#[cfg(test)]`, per house convention)

Observed (normative, cite claims):
- `observed_sum_by_subscript_matches_export` — `[1;2;1]`,`[10;20;30]`→`[40;20]`.
- `observed_output_is_double_column` — double column, no scalar collapse.
Documented-choice / unresolved (tagged `unresolved_choice_*`):
- scalar `vals` broadcast; `sz` extends length; `fillval` fills empty only;
  single-element collapse to scalar; multi-column rejected; custom-function
  rejected; `issparse` rejected.
Validation (`rejects_*`): non-positive / non-integer subscripts; vals length
mismatch; undersized `sz`. Plus `accepts_logical_vals_via_coercion`.

## Risks & rollback

- Risk: sibling features in Batch 5 touch other files concurrently — do not
  edit their files; re-run on transient compile/fmt noise.
- Risk: 1×1 collapse surprises callers expecting a `[1,1]` tensor — documented
  choice, consistent with `sub2ind`/`tensor_into_value`.
- Rollback: delete `accumarray.rs` and the single `mod` line; no existing file
  is otherwise modified.

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- accumarray` ·
`cargo fmt` then `cargo fmt --all -- --check` ·
`cargo clippy -p runmat-runtime --lib`
