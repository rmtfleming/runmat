# Implementation Plan — 019-computer

## Constitution Check

- I Specification-first: requirements cite approved claims — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB — PASS
- III Independence: mapping derived from Rust `std::env::consts`, not MATLAB
  internals — PASS
- IV Provenance: `provenance.md` complete; commit pin `e46587a` — PASS
- V Test-first: tests authored with the builtin file before verification;
  observed cases asserted through a pure mapping function — PASS
- VI Traceability: FR ↔ claim ↔ test mapping in `validation.md` — PASS
- VII Minimal scope: one builtin, no refactoring — PASS
- IX Compatibility: follows `introspection/` conventions (descriptor, macro
  registration, GPU/fusion metadata, inline tests) — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `computer` | `crates/runmat-runtime/src/builtins/introspection/computer.rs` (new) |
| Registration | add `pub mod computer;` to `crates/runmat-runtime/src/builtins/introspection/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Category choice: `introspection` — `computer` reports host/system
information, like `mfilename`/`which`/`version`-style introspection; it does
not touch the filesystem, so `io` is not appropriate.

Exemplars: `introspection/mfilename.rs` (optional text-option dispatch over
`args: Vec<Value>`), `logical/tests/iscell.rs` (GPU/fusion spec + descriptor
+ tiered test structure), `io/repl_fs/fileparts.rs` (char row vector
outputs, stable error identifiers), `strings/core/int2str.rs` (char output
type resolver, registry test).

## Design decisions

- Pure mapping `platform_strings(os, arch) -> (String, String)` returning
  the (upper-case identifier, lower-case arch string) pair:
  - `("linux", "x86_64")` → `("GLNXA64", "glnxa64")` — observed, normative
    [computer.platform-strings].
  - `("macos", "aarch64")` → `("MACA64", "maca64")`; `("macos", "x86_64")` →
    `("MACI64", "maci64")`; `("windows", "x86_64")` → `("PCWIN64", "win64")`
    — documented choices (unobserved).
  - Fallback: synthesized `OS-ARCH` upper/lower pair — documented choice; the
    output stays a non-empty char row vector on every platform RunMat builds
    for (including wasm32 and Linux aarch64).
- Live builtin resolves `platform_strings(std::env::consts::OS,
  std::env::consts::ARCH)` at call time; result is `Value::CharArray`
  (`CharArray::new_row`), the observed class.
- Argument handling via `args: Vec<Value>` (mfilename pattern): zero args →
  default identifier; one text arg matching `arch` (ASCII case-insensitive,
  char row vector or string scalar) → arch string; any other option or
  non-text arg → `RunMat:computer:InvalidOption`; >1 arg →
  `RunMat:computer:TooManyInputs` (all documented choices, FR-019-04).
- Output mode `Fixed` (single output); `computer.q-multi-output` unresolved →
  the `[str, maxsize, endian]` form is omitted (FR-019-05).
- Type resolver: local `computer_type` → `Type::String` (same as other char
  row vector producers, e.g. `int2str`).
- GPU/fusion metadata: host-only metadata query; `GpuOpKind::Custom`
  ("introspection"), empty precision list, `GatherImmediately`, not fusible
  (mirrors `fileparts`/`iscell` metadata-only pattern). No `accel` tag — no
  real device path.

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative (cite FR/claims in comments):
- `observed_default_linux_x86_64_maps_to_glnxa64` — pure mapping with the
  observed platform inputs reproduces the `default`/`arch` pair. Asserting
  the mapping (not the live host) keeps the observed test host-independent;
  documented in the test comment.
- `observed_live_builtin_glnxa64_pair` — `#[cfg(all(target_os = "linux",
  target_arch = "x86_64"))]`: end-to-end builtin calls reproduce
  `'GLNXA64'`/`'glnxa64'` as 1×7 char on the observed platform.
- `observed_output_class_char_row_vector` — on any host, both forms return a
  char row vector consistent with the mapping [computer.output-class].

Unresolved-tracking (tagged `unresolved_choice`):
- documented platform pairs (maca64/maci64/win64) via the mapping function;
- unmapped-platform fallback;
- `'ARCH'` case-insensitive acceptance; string-scalar option acceptance;
- unrecognized option (e.g. `'version'`) → InvalidOption;
- non-text option → InvalidOption; two args → TooManyInputs.

Registration/metadata: registry lookup (`builtin_function_by_name`),
descriptor shape (2 signatures, Fixed output), GPU/fusion spec names, type
resolver result.

## Risks & rollback

- Risk: another concurrent Tier B feature touches
  `introspection/mod.rs` — the change here is a single `pub mod computer;`
  line; conflicts are trivially mergeable.
- Risk: macro/WASM registry regeneration — handled by `builtin_path` on the
  macro; verified with `cargo check -p runmat-runtime`.
- Rollback: delete `computer.rs` and the one `mod` line (no other file
  changes).

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::introspection::computer` ·
`cargo fmt` then `cargo fmt --all -- --check`
