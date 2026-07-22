# Implementation Plan — 029-time-conversion

## Constitution Check

- I Specification-first: every requirement cites an approved claim id — PASS
- II Clean-room: sources = imported export + RunMat code + public-domain date
  algorithm; MATLAB never invoked — PASS
- III Independence: `etime` uses an independent civil-to-days conversion;
  `seconds` reuses RunMat's `duration` representation — PASS
- IV Provenance: `provenance.md` complete; commit pin `1b01912` recorded — PASS
- V Test-first: acceptance criteria fixed before implementation; observed +
  unresolved-choice tiers authored with the builtins — PASS
- VII Minimal scope: two tightly coupled time-conversion builtins; no
  unrelated refactoring — PASS
- IX Compatibility: follows the `datetime/` and `duration/` module
  conventions, `#[runtime_builtin]` registration, RunMat `Value` model — PASS
- X Licensing: date algorithm is public-domain (Hinnant); no GPL / MathWorks
  material — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `etime` | `crates/runmat-runtime/src/builtins/datetime/etime.rs` (new) |
| `seconds` | `crates/runmat-runtime/src/builtins/duration/seconds.rs` (new) |
| Registration | `+ pub mod etime;` in `datetime/mod.rs`; `+ pub mod seconds;` in `duration/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields on each builtin |

Exemplars: `datetime/datestr.rs` (datetime-category structure: GPU/fusion
specs, descriptor, error builder, civil date conversion, inline tests);
`duration/mod.rs` (duration `Value` construction via
`duration_object_from_days_tensor`); `logical/tests/iscell.rs` (file
structure and test tiers).

## Design decisions

- `etime(t2, t1)`: extract six components `[Y M D H MN S]` from each date
  vector; convert `(Y,M,D)` to a day count with an independent public-domain
  `days_from_civil` (Hinnant; inverse of `datestr`'s `civil_from_days`);
  total seconds = `days*86400 + H*3600 + MN*60 + S`; result =
  `seconds(t2) − seconds(t1)` as `Value::Num`. The epoch offset cancels in
  the difference, so the observed case reproduces regardless of anchor.
  type_resolver `num_scalar_type` → `Type::Num`.
- `seconds(x)`: convert numeric input to a tensor, divide each element by
  `86400` to obtain days, and build a `duration` via the duration module's
  `duration_object_from_days_tensor` (same representation as the `duration`
  constructor). Display `Format` = `"s"` (documented choice; unobserved).
- Both output mode `Fixed`, completion policy `Public`; GPU specs gather
  immediately (scalar / metadata style, mirroring `datestr`).
- Documented rejections: `etime` requires two six-element finite date
  vectors; `seconds` requires one numeric argument. Other classes/shapes are
  unobserved and rejected with a clear `InvalidInput` error.

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative (cite FR/claims in comments):
- etime: observed `one-second` → `1` (double 1×1).
- seconds: `seconds(5)` → duration, 1×1, magnitude 5 s (verified via the
  internal days tensor and the `"s"`-format display text `"5"`).

Unresolved-tracking (tagged `unresolved_choice`):
- etime: negative result when `t2 < t1`; cross-day span (90000 s);
  cross-year span (365 days, non-leap 2021→2022); malformed inputs / wrong
  arg counts error.
- seconds: array input maps element-wise; non-numeric / wrong arg counts
  error.

Descriptor smoke test on each builtin's primary signature label.

## Risks & rollback

- Risk: `duration` internal constructor signature differs from assumption —
  resolved by reading `duration/mod.rs`; `duration_object_from_days_tensor`
  is `pub(crate)` and reused directly.
- Risk: concurrent sibling features in the same batch cause transient
  cross-module compile errors — handled by re-running, not editing sibling
  files.
- Rollback: delete the two new files + the two `mod` lines (no existing file
  is modified beyond those additions).

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::datetime::etime builtins::duration` ·
`cargo fmt -p runmat-runtime` then `cargo fmt --all -- --check` ·
`cargo clippy -p runmat-runtime --lib`
