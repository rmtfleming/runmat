# Validation Report — 049-exceptions-warnings (specs 0.2.0) — 2026-10-30

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the cargo runner wrapper. The target directory was shared with other
agents. MATLAB was not invoked.

## Freshness check (T0, T9)

In `../matlab-interface-spec`, after `git fetch -q origin` (rc 0): `HEAD` =
`origin/main` = `1156c97`. `git log 8f72edb..HEAD` / `..origin/main` over
`specifications/{MException,lastwarn}` and `git log 1156c97..HEAD` /
`..origin/main` over `specifications/throw` were empty; `git status --short`
for the three directories was clean — at the start and again at the end of
the work (2026-10-30 08:52 UTC).

## Test-first evidence (T2–T3)

`exceptions_warnings.rs` was first written with the four builtins
registered and every body returning `RunMat:<fn>:Internal` "not yet
implemented", together with the complete test module; `policy.rs` got only
the `lastwarn` accessors needed to compile. `emission.rs`, `object/resolve.rs`
and `object/dispatch.rs` were not yet changed. Run
(`cargo test -p runmat-runtime --lib -- exceptions_warnings_tests …`):
**2 passed, 21 failed, 1 ignored**. The two passes were
`registration_discoverable_by_name` and `observed_mexception_rethrow_preserves`
(it uses only upstream `error` and `rethrow`, which already satisfy
MException.rethrow); every other behavioural test failed.

## Upstream changes (outside the new file)

| file | change | reason |
|---|---|---|
| `builtins/diagnostics/mod.rs` (**generated**) | one hand-added line `pub mod exceptions_warnings;` | registration; to be listed in `parallel_implementations/README.md` |
| `warnings/policy.rs` | new field `lastwarn: Option<RuntimeWarning>` with `lastwarn()` / `set_lastwarn()`; reset by `reset()` | lastwarn state (U-049-6) |
| `warnings/emission.rs` | `emit` stores every warning in `lastwarn` before the on/off/once/error decision | FR-049-10 [lastwarn.disabled] |
| `object/resolve.rs` | `Value::MException` member arm: `cause` → empty 0×1 cell; `throw` → raises the exception | FR-049-01 (no causes), FR-049-04 (`ME.throw`) |
| `object/dispatch.rs` | `Value::MException` method-call arm for `throw` with no arguments | FR-049-04 (`ME.throw()`) |

Upstream `warning` behaviour is unchanged: `last_warning` (used by
`warning('query'|'off'|…, 'last')`) still ignores suppressed warnings, so
upstream tests `off_suppresses_warning` / `once_only_emits_first_warning`
still pass.

## Commands and results (T8)

- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib --
  map_conformance_tests exceptions_warnings_tests`: **41 passed, 0 failed,
  2 ignored** (23 F49 tests pass; ignored F49 test:
  `deviation_observed_add_cause_and_thrown_cause`, D-049-1). Same filter in
  parallel: **41 passed, 0 failed, 2 ignored**.
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib --
  builtins::containers builtins::diagnostics warnings:: object::`: **224
  passed, 1 failed, 2 ignored**; the one failure is the pre-existing
  `containers_map_descriptor_includes_constructor_and_method_signatures`
  (label mismatch in upstream map descriptor; see 048 validation). No
  regressions in `error`, `warning`, `rethrow`, `getReport`, `assert`, the
  warning policy or the object layer.
- `rustfmt --edition 2021 --check` on `exceptions_warnings.rs`,
  `warnings/{policy,emission}.rs`, `object/{resolve,dispatch}.rs`: clean.
- clippy: see the final section.

## Traceability (claim → FR → test → result) (T9)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| MException.signature-primary, MException.output-class, MException.properties | FR-049-01 | `observed_mexception_construct_formats_message`, `observed_mexception_plain_percent_three_part`, `observed_mexception_member_reads` | PASS |
| MException.properties (no causes) | FR-049-01 | `deviation_observed_add_cause_and_thrown_cause` (`ME.cause` empty part) | ignored (D-049-1); `ME.cause` route added |
| MException.identifier-form | FR-049-02 | `observed_mexception_construct_formats_message`, `observed_mexception_plain_percent_three_part` | PASS |
| MException.error-bad-id | FR-049-03 | `rejects_mexception_bad_identifier` | PASS |
| MException.error-no-message | FR-049-03 | `rejects_mexception_identifier_only` | PASS |
| MException.throw-catch, throw.signature-primary, throw.identifier, throw.caught | FR-049-04 | `observed_mexception_throw_catch`, `observed_throw_direct_and_caught` | PASS |
| throw.caught (case dot) | FR-049-04 | `observed_throw_dot_syntax` (`ME.throw`, `ME.throw()`) | PASS |
| throw.formatted | FR-049-04 | `observed_throw_formatted_and_percent` | PASS |
| throw.no-id | FR-049-04 | `observed_throw_empty_identifier` | PASS |
| MException.rethrow, throw.rethrow | FR-049-05 | `observed_mexception_rethrow_preserves`, `observed_throw_of_caught_error` | PASS |
| MException.addcause, throw.cause | FR-049-06 | `deviation_add_cause_is_unsupported` (explicit error); observed behaviour: `deviation_observed_add_cause_and_thrown_cause` | PASS / ignored — **not met** (D-049-1) |
| lastwarn.signature-primary, lastwarn.after-warning | FR-049-07 | `observed_lastwarn_after_warning` | PASS |
| lastwarn.set | FR-049-08 | `observed_lastwarn_set_forms` | PASS |
| lastwarn.set-return | FR-049-09 | `observed_lastwarn_set_returns_previous` | PASS |
| lastwarn.disabled | FR-049-10 | `observed_lastwarn_disabled_warning_still_recorded` | PASS |
| lastwarn.no-id | FR-049-11 | `observed_lastwarn_no_identifier` | PASS |
| lastwarn.empty-class | FR-049-12 | `observed_lastwarn_cleared_outputs_are_0x0_char` | PASS |
| U-049-1 | FR-049-13 | `unresolved_choice_mexception_inputs` | PASS |
| throw.non-mexception (non-normative), U-049-2 | FR-049-13 | `unresolved_choice_throw_non_exception` | PASS |
| U-049-3 | FR-049-13 | `deviation_add_cause_is_unsupported` | PASS |
| U-049-4 | FR-049-13 | `unresolved_choice_lastwarn_inputs_and_outputs` | PASS |
| U-049-5 | FR-049-13 | `observed_lastwarn_no_identifier` | PASS |
| U-049-6 | FR-049-13 | upstream `warning` tests (`off_suppresses_warning`, …) still pass in the diagnostics suite | PASS |
| registration | SC-049-2 | `registration_discoverable_by_name` | PASS |
| *.q-stack, *.q-warning-coverage | FR-049-14 | not asserted | not tested |

Every FR maps to at least one test; FR-049-06 is reported as not met.

## Error identifiers

`RunMat:MException:badIdentifier`, `RunMat:minrhs`,
`RunMat:MException:InvalidInput`, `RunMat:MException:TooManyOutputs`,
`RunMat:throw:UndefinedFunction`, `RunMat:throw:InvalidInput`,
`RunMat:addCause:UnsupportedOption`, `RunMat:addCause:InvalidInput`,
`RunMat:lastwarn:InvalidInput`, `RunMat:lastwarn:UnsupportedOption`,
`RunMat:lastwarn:TooManyOutputs`. A thrown exception keeps its own
identifier (including `''`).

## Deviations

- **D-049-1** Causes (`addCause`, `throw.cause`, `numel(ME.cause)` after
  addCause) are not supported: `runmat_value::MException` has no cause list
  and `RuntimeError` cannot carry one into `catch`. Needs an architecture
  decision (value type in `runmat-value`, value codec, error propagation).
  `addCause` raises `RunMat:addCause:UnsupportedOption`; `ME.cause` is an
  empty 0×1 cell. Ignored test holds the observed behaviour.
- **D-049-2** `ME.throw` / `ME.throw()` are handled at the runtime object
  layer; the VM's compilation of a bare `ME.throw;` statement into these
  routes is not exercised here (VM integration tests not run).

## Notes

- Hand-added line in a generated parent: `pub mod exceptions_warnings;` in
  `crates/runmat-runtime/src/builtins/diagnostics/mod.rs`.
- Integer-capability metadata was not declared.
- No MathWorks material was consulted; MATLAB was not invoked.

## Final checks

- `cargo clippy -p runmat-runtime --lib --tests --message-format short`
  after the fix: **no findings anywhere in the crate** (`Finished dev
  profile`), lib and tests targets.
- Re-run after the clippy fix: `RUST_TEST_THREADS=1 cargo test -p
  runmat-runtime --lib -- map_conformance_tests exceptions_warnings_tests`
  → **41 passed, 0 failed, 2 ignored**.
- Final freshness check (2026-10-30 08:52 UTC): clean, as above.
