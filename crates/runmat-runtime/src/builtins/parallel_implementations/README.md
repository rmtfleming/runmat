# parallel_implementations

Independently-developed ("clean-room") RunMat builtin implementations, written against the
[`matlab-interface-spec`](https://github.com/Digital-Metabolic-Twin-Centre/matlab-interface-spec)
black-box specifications, that duplicate builtins **RunMat `0-6-0` already provides as the default**.

## Why this module exists

Over the `0-6-0` development cycle the RunMat maintainers independently implemented most of the
builtins that were also implemented on the clean-room branch. Rather than discard one of the two
independent implementations, this repository keeps **both**:

- The `0-6-0` implementation stays the **registered default** — it owns dispatch.
- The clean-room implementation is retained **here**, compiled but **not registered**: each file's
  `#[runtime_builtin]`, `register_gpu_spec`, and `register_fusion_spec` attributes are stripped, so
  it never enters the inventory registry and never shadows the default.

Keeping two independent implementations of the same black-box spec enables **differential
cross-validation**: each carries its own spec-derived unit tests (retained verbatim), and the two
can be compared against the same specification observations.

The clean-room unit tests are retained but **gated behind the `parallel_xval` cargo feature** so the
default build/test stays green:

```
cargo test -p runmat-runtime --features parallel_xval
```

A handful of these tests currently need small updates for `0-6-0` drift (they assert on
type-resolver internals and the `SparseTensor` representation, which changed since the branch was
authored) — fix those when enabling the feature. The implementations themselves compile in the
default build.

## Contents

**Parallel (unregistered) implementations — 34:**

`accumarray`, `append`, `bounds`, `datestr`, `dec2bin`, `display`
`eps`, `etime`, `fileparts`, `full`, `histc`, `int2str`
`iscell`, `iscellstr`, `iscolumn`, `isfile`, `isfolder`, `isobject`
`isrow`, `issparse`, `istable`, `matches`, `nonzeros`, `normalize`
`num2cell`, `spdiags`, `speye`, `str2num`, `strncmpi`, `strtok`
`system`, `verLessThan`, `version`, `what`

**Excluded (2)** — retained as files but not compiled/wired, because they depend on
sibling-subsystem internals not vendored here (`saveas` → plotting figure state/print; `seconds` →
duration-subsystem helpers). Re-enable by vendoring those helpers into `support.rs` and adding the
module back to `mod.rs`.

**Genuinely new builtins (17)** — implemented here because `0-6-0` does *not* provide them;
registered normally in their natural category modules (not in this directory):

`colamd`, `computer`, `dissect`, `fields`, `getReport`, `inputParser`
`isdir`, `isstr`, `isstruct`, `license`, `methods`, `properties`
`psi`, `spalloc`, `strmatch`, `urlread`, `ver`

> `eps` was initially treated as new but 0-6-0 provides it as a primitive **constant**; the clean-room
> **function** form (`eps(x)`) is therefore kept as a parallel implementation instead of registered.

## Support code

- `support.rs` — small self-contained helpers (`scalar_f64`, `size_from_f64`, `is_rooted_path`)
  vendored from the original branch so the parallel files are self-contained.
- `text_utils.rs` — vendored verbatim for the parallel `matches` implementation.

A few registered builtins (`ver`, `license`, `isdir`) reuse shared helpers from the matching
parallel copies (`version`, `isfolder`), since those are the implementations they were written
against.
