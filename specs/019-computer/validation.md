# Validation Report — 019-computer — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (Finished, 20.73s after sibling
  build-lock wait)
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::introspection::computer`
  — **12 passed, 0 failed** (6705 filtered out):
  `computer_is_registered`, `computer_type_is_string`,
  `descriptor_and_spec_metadata_consistent`,
  `observed_default_linux_x86_64_maps_to_glnxa64`,
  `observed_live_builtin_glnxa64_pair`,
  `observed_output_class_char_row_vector`,
  `unresolved_choice_arch_option_case_insensitive`,
  `unresolved_choice_documented_platform_pairs`,
  `unresolved_choice_non_text_option_errors`,
  `unresolved_choice_too_many_inputs_errors`,
  `unresolved_choice_unmapped_platform_fallback`,
  `unresolved_choice_unrecognized_option_errors` — all `ok`
- `cargo fmt` then `cargo fmt --all -- --check` — clean

## Traceability (claim → FR → test → result)

| Claim | FR | Test (`builtins::introspection::computer::tests::…`) | Result |
|---|---|---|---|
| computer.signature-primary, computer.platform-strings, computer.output-class | FR-019-01 | `observed_default_linux_x86_64_maps_to_glnxa64`, `observed_live_builtin_glnxa64_pair`, `observed_output_class_char_row_vector` | PASS |
| computer.platform-strings, computer.output-class ('arch' form) | FR-019-02 | `observed_default_linux_x86_64_maps_to_glnxa64`, `observed_live_builtin_glnxa64_pair`, `observed_output_class_char_row_vector` | PASS |
| (documented choice — non-observed platforms) | FR-019-03 | `unresolved_choice_documented_platform_pairs`, `unresolved_choice_unmapped_platform_fallback` | PASS (non-normative) |
| (documented choice — option/arity handling) | FR-019-04 | `unresolved_choice_arch_option_case_insensitive`, `unresolved_choice_unrecognized_option_errors`, `unresolved_choice_non_text_option_errors`, `unresolved_choice_too_many_inputs_errors` | PASS (non-normative) |
| computer.q-multi-output (unresolved → omitted) | FR-019-05 | `descriptor_and_spec_metadata_consistent` (Fixed single output, 2 signatures) | PASS |
| — | SC-019-2 | `computer_is_registered`, `computer_type_is_string` | PASS |

Both observed cases reproduce exactly: `computer` → `'GLNXA64'` and
`computer('arch')` → `'glnxa64'`, each class char, size 1×7
(`CharArray` rows 1, cols 7).

## Deviations and notes

- Host independence (SC-019-1): the normative observed pair is asserted
  through the pure mapping function `platform_strings("linux", "x86_64")`,
  so the observed tests pass on any build host. The end-to-end test
  `observed_live_builtin_glnxa64_pair` is `#[cfg(all(target_os = "linux",
  target_arch = "x86_64"))]` and ran (and passed) on this host, which is the
  observed platform.
- Identifier pairs for macOS (MACA64/maca64, MACI64/maci64) and Windows
  (PCWIN64/win64) plus the synthesized `OS-ARCH` fallback for unmapped
  platforms are documented independent choices — tagged `unresolved_choice`
  in tests and NOT asserted MATLAB-conformant.
- The `[str, maxsize, endian]` multi-output form is omitted (descriptor
  output mode Fixed), per unresolved question `computer.q-multi-output`.
- Registration is via the `#[runtime_builtin]` inventory macro with
  `builtin_path` (native + WASM registries), verified through
  `builtin_function_by_name("computer")`.

## Unresolved (returned to specification side)

`computer.q-multi-output` (`[str, maxsize, endian]` form); observation
coverage for non-GLNXA64 platforms and for option-argument edge cases
(case variants, invalid options).
