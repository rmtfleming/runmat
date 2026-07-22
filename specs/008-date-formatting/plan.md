# Implementation Plan — 008-date-formatting

## Constitution Check

- I Specification-first: requirements cite approved claims — PASS
- II Clean-room: sources = imported export + RunMat code + public-domain
  date algorithm; no MATLAB invoked — PASS
- III Independence: date math is an independent Rust implementation of a
  published public-domain algorithm; epoch anchored from the observation,
  not from MATLAB internals — PASS
- IV Provenance: `provenance.md` complete; interim pin waiver recorded;
  external algorithm source recorded — PASS
- V Test-first: observed-case tests authored with the module before the
  formatting logic was finalised (tasks T2 before T3–T4) — PASS
- VII Minimal scope: one builtin, one new file plus a one-line `mod`
  registration; no refactoring — PASS
- IX Compatibility: follows `logical/tests/iscell.rs` and
  `io/repl_fs/fileparts.rs` house conventions — PASS
- X Licensing: only public-domain algorithm material; recorded in
  `provenance.md` — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `datestr` | `crates/runmat-runtime/src/builtins/datetime/datestr.rs` (new) |
| Registration | `pub mod datestr;` in `crates/runmat-runtime/src/builtins/datetime/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplars: `logical/tests/iscell.rs` (GPU/fusion specs, descriptor, error
builders, inline test module) and `io/repl_fs/fileparts.rs` (text output,
error identifiers). The `datetime` category previously had no submodules;
`datestr.rs` is its first, registered exactly like `logical/tests/*`.

## Design decisions

- Input: finite scalar numeric serial date number (`Value::Num`,
  `Value::Int`, 1×1 `Value::Tensor`); GPU inputs gathered via
  `gather_if_needed_async` (GPU spec residency `GatherImmediately`). Date
  text / date vectors / non-scalars are `pending` in the export → rejected
  with `RunMat:datestr:InvalidInput` (documented choice).
- Serial→date: `split_serial` separates whole day and fractional day
  (rounded to nearest second, midnight carry); `civil_from_days` converts
  days-since-1970-01-01 to proleptic Gregorian y/m/d (public-domain Hinnant
  algorithm, independent implementation); epoch constant
  `UNIX_EPOCH_SERIAL_DAY = 719529` anchored so 738885 → 30-Dec-2022.
- Formatting: longest-match token scan over the format text. Tokens:
  `yyyy` (4-digit year), `mmm` (English Jan..Dec), `mm` (2-digit month),
  `dd` (2-digit day), plus documented-choice `HH`/`MM`/`SS`
  (hour/minute/second). Non-alphabetic characters pass through literally;
  any other alphabetic character → `RunMat:datestr:InvalidFormat`.
- Default format `dd-mmm-yyyy` (from observed case `serial-default`).
- Output: `Value::CharArray` row vector always (datestr.output-class);
  output mode `Fixed`, single output.

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative (cite FR/claims in comments):
- `serial-default`: datestr(738885) → '30-Dec-2022' (1×11 char).
- `serial-format`: datestr(738885, 'yyyy-mm-dd') → '2022-12-30' (1×10 char).
- 1×1 tensor serial input → same as scalar.

Unresolved-tracking (tagged `unresolved_choice` in name):
- epoch anchoring (serial 1 → '01-Jan-0000', 719529 → '01-Jan-1970');
- time-of-day tokens on fractional serial (738885.5 → '… 12:00:00');
- second rounding with midnight carry;
- string-scalar format argument;
- unknown token → error; invalid date inputs → error; argument count →
  error.

## Risks & rollback

- Risk: parallel batch agents (005–009) share the working tree, so
  crate-wide test compilation can transiently fail on other features'
  files — mitigated by re-running the verification commands once the tree
  settles.
- Risk: WASM registry regeneration — handled by `build.rs` scanning
  `builtin_path`; verified via `cargo check -p runmat-runtime`.
- Rollback: delete `datestr.rs` and the single `pub mod datestr;` line.

## Verification commands

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::datetime::datestr`
