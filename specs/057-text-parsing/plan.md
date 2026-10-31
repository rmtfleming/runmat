# Implementation Plan — 057-text-parsing

## Constitution Check

- I Specification-first: FR-057-01..09, 13, 14, 16..19, 21, 22 cite the 19
  approved normative claims; the non-normative `empty-matches` claim is
  handled by the labelled choice FR-057-27; FR-057-25..31 are tagged
  documented choices or unresolved. PASS.
- II Clean-room: inputs are the imported exports and RunMat code. MATLAB was
  not invoked. PASS.
- III Independence: a whitespace/delimiter splitter with round-robin
  assignment; a regex quantifier over RunMat's existing pattern objects.
  PASS.
- IV Provenance: `provenance.md` pins `1156c97` (strread) and `fc9f804`
  (asManyOfPattern), freshness-checked, 20 claim ids verified. PASS.
- V Test-first: tests are written with stub bodies, the failing run is
  recorded in `validation.md`, then the stubs are removed. PASS.
- VII Minimal scope: two new files and one module line in each of two
  **hand-written** parents (`io/filetext/mod.rs`, `strings/core/mod.rs`).
  `textread.rs` is not edited (D-057-2). The missing `extract`,
  `alphanumericsPattern` and pattern `|` are documented, not added
  (D-057-3). PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptors; `strread`
  `accel = "cpu"`, `asManyOfPattern` `accel = "metadata"` (like the other
  pattern constructors), category `strings/pattern`. Pattern objects via
  `compat::pattern_object`/`pattern_regex`. No integer-capability metadata.
  PASS.

## Placement

| Builtin | File | Parent (hand-written) |
|---|---|---|
| `strread` | `crates/runmat-runtime/src/builtins/io/filetext/strread.rs` | `io/filetext/mod.rs`, next to `textread` |
| `asManyOfPattern` | `crates/runmat-runtime/src/builtins/strings/core/as_many_of_pattern.rs` | `strings/core/mod.rs`, next to `patterns.rs`/`compat.rs` (where `pattern`, `digitsPattern`, `lettersPattern` live) |

## Design

### strread
1. Gather all inputs. No input → `InvalidInput`. `str`: numeric →
   `SecondInputFilenameOrString`; char row (or `''`) → text; anything else
   → `UnsupportedOption`.
2. Format (char row): a sequence of `%s`/`%d`/`%f`, whitespace allowed
   between them; anything else → `UnsupportedOption`.
3. Options: `'delimiter'` + one non-whitespace char; anything else →
   `UnsupportedOption`; a name without a value → `InvalidInput`.
4. Requested outputs are checked against the number of conversions (1 with
   no format) before reading → `TooManyOutputs`.
5. Tokens: `split_whitespace`, or split at the delimiter and trim each field
   (empty field → `UnsupportedOption`; blank text → no fields).
6. No format: parse every field as `f64` → 1×n row. With a format: output j
   collects fields j, j+k, j+2k, …; `%s` → n_j×1 cell of char rows,
   `%d`/`%f` → n_j×1 double. Unreadable numeric field → `TroubleReading`.
7. Return the first output, or an `OutputList` of the requested ones.

### asManyOfPattern
1. At most one output; one to three inputs. Gather.
2. `p`: pattern object → its regex; string scalar / char row → escaped text
   (`pattern_regex`); numeric (incl. sparse/complex) →
   `MustBeTextOrPattern`; other → `UnsupportedOption`.
3. Counts: real numeric scalar; `max = +Inf` → unbounded; otherwise must be
   a non-negative integer (else `MustBeNonnegativeInteger`); `min > max` →
   `InvalidLimitRange`.
4. Return `pattern_object("(?:" + regex + ")" + quantifier)`.

## Test plan

Inline `#[cfg(test)]` modules `strread_tests` and
`as_many_of_pattern_tests`, calling through `crate::call_builtin_async`
(multi-output and output-count tests call the builtin function directly
under `push_output_count`).

- One `observed_*` test per observed case: 15 for `strread`, 12 normative
  for `asManyOfPattern`, which use the D-057-3 stand-ins for `extract` and
  `alphanumericsPattern | "_"`, and upstream `matches`, `contains`,
  `class`, `lettersPattern`, `digitsPattern`.
- The two non-normative `asManyOfPattern` cases form one
  `documented_choice_empty_matches` test.
- `documented_choice_*`/`unresolved_*` tests cover FR-057-25..31.

## Risks & rollback

- If upstream later adds `extract` or pattern `|`, the stand-ins in the
  tests should be replaced by the real builtins.
- Rollback: delete the two files and their module lines.

## Verification commands

- `cargo test -p runmat-runtime --lib -- strread_tests as_many_of_pattern_tests`
- `cargo test -p runmat-runtime --lib -- builtins::strings builtins::io::filetext` (regressions)
- `rustfmt --edition 2021 --check <files>`
- `cargo clippy -p runmat-runtime --lib --tests`, filtered to the two files
