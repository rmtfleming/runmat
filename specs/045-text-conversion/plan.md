# Implementation Plan — 045-text-conversion

## Constitution Check

- I Specification-first: FR-045-01..17 cite the 27 approved claims.
  FR-045-18..28 are tagged as documented choices or unresolved. PASS.
- II Clean-room: the only inputs are the imported exports and RunMat code.
  MATLAB was not invoked. PASS.
- III Independence: a character substitution table, textbook repeated
  division, and a prefix filter. PASS.
- IV Provenance: `provenance.md` is pinned to `c65e238` and was
  freshness-checked. All 27 claim ids were verified. PASS.
- V Test-first: tests are written before the bodies, and the failing run is
  recorded in `validation.md`. PASS.
- VII Minimal scope: three new files, plus one `pub mod` line each in two
  **hand-written** parents (`strings/regex/mod.rs` and
  `strings/core/mod.rs`; neither carries the "@generated … Do not edit"
  header). No generated parent is edited. PASS.
- IX Compatibility:
  - Registration uses `#[runtime_builtin]` with a descriptor and
    `accel = "cpu"`.
  - Inputs are gathered from the GPU with `gather_if_needed_async`.
  - No integer-capability metadata is added. PASS.

## Placement

| Builtin | File | Parent (hand-written) |
|---|---|---|
| `regexptranslate` | `crates/runmat-runtime/src/builtins/strings/regex/regexptranslate.rs` | `strings/regex/mod.rs` (next to `regexprep`) |
| `dec2base` | `crates/runmat-runtime/src/builtins/strings/core/dec2base.rs` | `strings/core/mod.rs` |
| `validatestring` | `crates/runmat-runtime/src/builtins/strings/core/validatestring.rs` | `strings/core/mod.rs` |

`dec2base`: the registered `dec2bin` lives in `comms/binary_conversion.rs`,
whose parent `comms/mod.rs` is generated. The parallel copy
`parallel_implementations/dec2bin.rs` records its original home as
`strings/core/dec2bin.rs`. `strings/core/` is therefore the natural
hand-written home.

## Design

### regexptranslate
1. Check arity (2). Parse `op` from a char row or string scalar:
   - `escape` or `wildcard` is accepted;
   - a case variant of either → `UnsupportedOption`;
   - any other text → `RunMat:REGEXP:invalidOptions`;
   - non-text → `UnsupportedOption`.
2. Translate a `&str` one character at a time:
   - escape: put `\` before characters in `.$^*+?()[]{}|\-`;
   - wildcard: `*` → `.*`, `?` → `.`, `.` → `\.`.
3. Shapes:
   - char row → char row (an empty result is 0×0);
   - string → string;
   - string array → same shape;
   - cell → same shape, char-row elements only.

### dec2base
1. Arity 2–3. Gather inputs.
2. Base:
   - a numeric or integer scalar that is an integer in 2..=36;
   - otherwise `SecondArg`;
   - a char base: two-character row → `SecondArg`, else `UnsupportedOption`;
   - string or logical → `UnsupportedOption`.
3. `n` (optional): a non-negative integer real scalar, otherwise
   `UnsupportedOption`.
4. Elements of `d`:
   - double scalar or tensor (F64), or integer class (scalar or tensor),
     taken in column-major order;
   - each must be finite, ≥ 0 and integer-valued, otherwise `FirstArg`;
   - a value > 2^52 → `UnsupportedOption`.
5. Convert each `u64` by repeated division. Width = max(n, longest). Left-pad
   with `'0'` and build a `rows × width` `CharArray` (0×0 when `d` is empty).

### validatestring
1. Arity:
   - fewer than 2 → `NotEnoughInputs`;
   - more than 4 → `UnsupportedOption`;
   - trailing funcName and varName must be text.
2. Parse the list:
   - a cell of char rows (`Char` kind);
   - a string scalar or array (`String` kind);
   - anything else → `invalidStringList`.
3. Parse `txt` as a char row or string scalar. Anything else, or empty text,
   → `unrecognizedStringChoice`.
4. Matches are the options whose lower-case form starts with the
   lower-case text:
   - none → `unrecognizedStringChoice`;
   - otherwise pick the first shortest match. If it is a prefix of every
     other match (compared case-insensitively), return it. If not →
     `ambiguousStringChoice`.
5. Return a char row (`Char` kind) or a string scalar (`String` kind).

## Test plan

Tests are inline `#[cfg(test)]` modules `regexptranslate_tests`,
`dec2base_tests` and `validatestring_tests`, calling through
`crate::call_builtin_async`.

- Each observed case has one `observed_*` test: 13 + 17 + 13.
- `documented_choice_*` and `unresolved_*` tests cover FR-045-18..28.

## Risks & rollback

- No generated parent is touched, so regeneration of the module
  composition cannot drop these modules.
- Rollback: delete the three files and their `pub mod` lines.

## Verification commands

- `cargo test -p runmat-runtime --lib -- regexptranslate_tests dec2base_tests validatestring_tests`
- `cargo test -p runmat-runtime --lib -- builtins::strings` (regressions)
- `rustfmt --edition 2021 --check <files>`
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`,
  filtered to the three files
