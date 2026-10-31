# Implementation Plan — 064-file-io-n5

## Constitution Check

- I Specification-first: FR-064-01..16 cite the 17 approved normative
  claims; FR-064-20..26 are tagged documented choices or unresolved. PASS.
- II Clean-room: inputs are the imported exports and RunMat code. MATLAB was
  not invoked. PASS.
- III Independence: a format loop over RunMat's own `sscanf` scanner; URL
  routing over RunMat's filesystem and transport layers. PASS.
- IV Provenance: `provenance.md` pins `1c91aa1`, freshness-checked, 17
  claim ids verified. PASS.
- V Test-first: tests are written with stub bodies, the failing run is
  recorded in `validation.md`, then the stubs are removed. PASS.
- VII Minimal scope: two new files, one module line in each of two
  **hand-written** parents (`io/filetext/mod.rs`, `io/http/mod.rs`), and
  seven visibility changes (`pub(crate)`) in `strings/core/compat.rs`
  (D-064-2) with no body change. PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptors; `fscanf`
  `accel = "cpu"` (category `io/filetext`), `urlwrite` `accel = "sink"`
  (category `io/http`, like `urlread`). No integer-capability metadata.
  PASS.

## Placement

| Builtin | File | Parent (hand-written) |
|---|---|---|
| `fscanf` | `crates/runmat-runtime/src/builtins/io/filetext/fscanf.rs` | `io/filetext/mod.rs`, next to `fread`/`fgetl` |
| `urlwrite` | `crates/runmat-runtime/src/builtins/io/http/urlwrite.rs` | `io/http/mod.rs`, next to `urlread` |

## Design

### fscanf
1. More than two requested outputs → `TooManyOutputs`. Gather inputs. Fewer
   than two → `RunMat:minrhs`; more than three → `UnsupportedOption`.
2. fid: double integer scalar; 0–2 → `UnsupportedOption`; otherwise it must
   be in the `fopen` registry and readable, else `RunMat:FileIO:InvalidFid`.
3. Format: char row / string scalar (else `InvalidInput`), parsed with
   `compat::parse_scan_format`; `%l…`, unknown conversions, no conversions
   → `UnsupportedOption`.
4. Size: n, `Inf`, `[m n]`, `[m Inf]` (else `InvalidInput`).
5. Lock the registry handle, read from the current position to the end,
   decode with the fid's encoding (`helpers::decode_bytes`).
6. Loop over the format: whitespace token skips whitespace, a literal must
   match, a conversion skips leading whitespace (not for `%c`) and calls
   `compat::scan_one`. Stop on the first failure, at the end of the text, on
   no progress, or when the size limit is reached. Count non-suppressed
   conversions.
7. Seek the file to just after the consumed text; mark EOF when consumed.
8. Shape: no value → 0×0; `[m …]` → m×ceil(k/m) zero-padded, column-major;
   else char 1×k (all conversions `%c`/`%s`) or double k×1.
9. Return A, or an `OutputList` [A, count].

### urlwrite
1. More than two outputs → `TooManyOutputs`. Fewer than two inputs →
   `RunMat:minrhs`; more than two → `UnsupportedOption`; non-text →
   `InvalidInput`.
2. Destination folder (parent of the file name, `.` if none) must be an
   existing folder, else `InvalidOutputLocation` (both output modes).
3. Parse the URL; parse failure or a scheme other than file/http/https →
   `InvalidUrl`. `file` → read the path (`FileNotFound` on failure). http(s)
   → shared transport GET (`ConnectionFailed` on failure).
4. Write the bytes (`InvalidOutputLocation` on failure).
5. Success → `f` = file name as given, `status` = 1. Source/URL failure
   with two outputs → `f = ''` (0×0 char), `status = 0`; with one output
   the error is raised.

## Test plan

Inline `#[cfg(test)]` modules `fscanf_tests` and `urlwrite_tests`. Calls go
directly to the builtin function under `push_output_count` (two outputs and
the too-many-outputs checks), plus one dispatcher call per builtin
(`crate::call_builtin_async`). File tests use `tempfile` directories and the
file-registry test lock; the cwd is never changed. The http tests use a
loopback listener on 127.0.0.1 (no network).

- One `observed_*` test per observed case (two bad-dir cases share one
  test): 16 for `fscanf`, 8 tests covering 9 cases for `urlwrite`.
- `documented_choice_*` tests cover FR-064-20..25.

## Risks & rollback

- `compat.rs` is shared with `sscanf`; only visibility changed, so `sscanf`
  behaviour cannot change. The `builtins::strings` suite is run as a
  regression check.
- Rollback: delete the two files, their module lines and the seven
  `pub(crate)` qualifiers.

## Verification commands

- `cargo test -p runmat-runtime --lib -- fscanf_tests urlwrite_tests`
- `cargo test -p runmat-runtime --lib -- builtins::io::filetext builtins::io::http builtins::strings::core` (regressions)
- `rustfmt --edition 2021 --check <files>`
- `cargo clippy -p runmat-runtime --lib --tests`, filtered to the F64 files
