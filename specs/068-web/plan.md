# Implementation Plan — 068-web

## Constitution Check

- I Specification-first: FR-068-01..06 cite the 6 approved normative
  claims; FR-068-07..09 record the 3 non-normative claims; FR-068-10..15
  are tagged documented choices or unresolved. PASS.
- II Clean-room: the only inputs are the imported export and RunMat code.
  MATLAB was not invoked. PASS.
- III Independence: request building, URL normalisation with the `url`
  crate, and the platform URL opener. PASS.
- IV Provenance: `provenance.md` pins `e364206`, freshness-checked, 9 claim
  ids verified. PASS.
- V Test-first: stub line, failing run recorded, then implemented. PASS.
- VII Minimal scope: one new file plus one module line in the hand-written
  `io/http/mod.rs`. No upstream code changed (the `close` limitation is a
  recorded deviation, D-068-4). PASS.
- IX Compatibility: `#[runtime_builtin]` with a descriptor,
  `accel = "sink"`, `suppress_auto_output = true`; inputs gathered. PASS.

## Placement

`crates/runmat-runtime/src/builtins/io/http/web.rs`, parent
`io/http/mod.rs` (hand-written) next to `urlread`/`urlwrite`.

## Design

1. Output count > 2 → `RunMat:web:TooManyOutputs`.
2. Each argument gathered and read as text: char row, empty char, string,
   1×1 string array. Cell, multi-row char, larger string arrays →
   `RunMat:web:UnsupportedOption`; anything else →
   `RunMat:string:MustBeCharCellArrayOrString`.
3. Request: URL (scheme kept; plain path → absolute `file://` URL), target
   (`-browser` → system browser, else RunMat browser), flags `-new`,
   `-notoolbar`, `-noaddressbox`, unrecognised options kept.
4. Launch through the global launcher slot (`set_web_launcher`); default
   launcher: system browser via the platform opener when a display exists;
   `cfg(test)`: no-op. Result ignored (stat 0).
5. Outputs: none / `stat` / `stat, h` (`h` = structure `Type`, `URL`; `[]`
   with `-browser`).

## Test plan

Inline `#[cfg(test)] mod web_tests`. Every test installs a recording
launcher (a `Recorder` guard that serialises on a test mutex and restores
the default on drop) and calls `web_builtin` directly under
`push_output_count`. Pages are written to `tempfile` directories; no test
changes the current folder. One `observed_*` test per observed case (14),
`documented_choice_*` tests for FR-068-10..14, an `#[ignore]`d deviation test
for `close(h)`, and an `unresolved_*` test.

## Risks & rollback

- A test that forgot to install the recorder would still not launch
  anything (the default launcher is a no-op under `cfg(test)`).
- Rollback: delete the file and its module line.

## Verification commands

- `cargo test -p runmat-runtime --lib -- web_tests`
- `cargo test -p runmat-runtime --lib -- builtins::io::http`
- `rustfmt --edition 2021 --check crates/runmat-runtime/src/builtins/io/http/web.rs`
- `cargo clippy -p runmat-runtime --lib --tests`, filtered to the file
