# Validation Report — 027-sparse-ordering — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (exit 0).
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::sparse::colamd builtins::math::sparse::dissect`
  — **23 passed; 0 failed; 0 ignored** (11 `colamd` + 12 `dissect`).
- `cargo fmt` then `cargo fmt --all -- --check` — clean (exit 0).

## Non-assertion of the specific permutation values *(binding)*

The approved exports state the ordering "is algorithm-dependent and is not
asserted" (`colamd.q-values`, `dissect.q-values`). Accordingly, the **normative**
tests (`observed_input_returns_valid_permutation`, the dense / larger /
rectangular / empty cases) assert **only** three things:

1. class `double` (result is `Value::Tensor`, i.e. RunMat's double matrix),
2. size `1×n`, and
3. that the vector is a **valid permutation of `1..n`** (each index exactly
   once — checked by `assert_valid_permutation`).

The **specific permutation values are never asserted normatively.** Each
implementation's own output on the observed input is recorded only in a
`documented_choice_ordering_on_observed_input` test, explicitly flagged
non-normative.

## This implementation's ordering outputs (non-normative record)

| function | observed input | observed MATLAB value (non-normative) | THIS implementation's output | valid permutation of 1..3? |
|---|---|---|---|---|
| `colamd` | `sparse([1 0 1; 0 1 0; 1 0 1])` | `[2 1 3]` | `[2 1 3]` | yes |
| `dissect` | `sparse([2 1 0; 1 2 1; 0 1 2])` | `[3 1 2]` | `[3 1 2]` | yes |

Both independent heuristics happen to coincide with the observed MATLAB values
on these 3×3 inputs. **This coincidence is incidental and is NOT relied upon:**
the heuristics (minimum-degree / level-structure nested dissection) are derived
from public literature, not from MATLAB/SuiteSparse/METIS, and the tests treat
the values as non-normative documented choices.

## Traceability (claim → FR → test → result)

| Claim(s) | FR | Test(s) | Result |
|---|---|---|---|
| colamd.signature-primary, colamd.output-class, colamd.permutation-vector | FR-027-01, FR-027-02 | `colamd::observed_input_returns_valid_permutation`, `dense_input_is_accepted_and_valid`, `rectangular_input_valid_permutation`, `empty_matrix_yields_empty_permutation` | PASS (normative: class/size/valid-permutation only) |
| dissect.signature-primary, dissect.output-class, dissect.permutation-vector | FR-027-01, FR-027-03 | `dissect::observed_input_returns_valid_permutation`, `dense_input_is_accepted_and_valid`, `larger_symmetric_input_valid_permutation`, `empty_matrix_yields_empty_permutation` | PASS (normative: class/size/valid-permutation only) |
| colamd.q-values / dissect.q-values (non-normative) | FR-027-04 | `colamd::documented_choice_ordering_on_observed_input`, `dissect::documented_choice_ordering_on_observed_input`, `*::documented_choice_scalar_input` | PASS (documented choice; values NOT asserted as MATLAB-conformant) |
| unobserved input classes | FR-027-05 | `*::documented_choice_logical_input_accepted`, `dissect::documented_choice_non_square_input_accepted`, `*::documented_choice_non_matrix_input_errors` | PASS (documented choice; non-normative) |
| registration / architecture (Principle IX) | SC-027-2 | `*::descriptor_is_registered`, `*::gpu_and_fusion_specs_are_custom_and_non_fusible`, `*::type_resolver_is_row_vector` | PASS |

Full test roster (23):

`colamd`: observed_input_returns_valid_permutation,
documented_choice_ordering_on_observed_input, dense_input_is_accepted_and_valid,
rectangular_input_valid_permutation, empty_matrix_yields_empty_permutation,
documented_choice_scalar_input, documented_choice_logical_input_accepted,
documented_choice_non_matrix_input_errors, descriptor_is_registered,
gpu_and_fusion_specs_are_custom_and_non_fusible, type_resolver_is_row_vector.

`dissect`: observed_input_returns_valid_permutation,
documented_choice_ordering_on_observed_input, dense_input_is_accepted_and_valid,
larger_symmetric_input_valid_permutation, empty_matrix_yields_empty_permutation,
documented_choice_scalar_input, documented_choice_non_square_input_accepted,
documented_choice_logical_input_accepted,
documented_choice_non_matrix_input_errors, descriptor_is_registered,
gpu_and_fusion_specs_are_custom_and_non_fusible, type_resolver_is_row_vector.

## Deviations and notes

- **Independent heuristics (Constitution Principle III / gate constraint).**
  `colamd` uses classic minimum-degree ordering on the column-intersection
  graph (Tinney & Walker 1967; George & Liu 1981); `dissect` uses recursive
  nested dissection via level-structure bisection with a pseudo-peripheral root
  (George 1973; George & Liu 1981; Gibbs–Poole–Stockmeyer 1976). Neither
  reproduces SuiteSparse COLAMD or MATLAB's METIS-based nested dissection.
  Sources recorded in `provenance.md` and the module doc comments (Principle X).
- **Result shape.** Output is a 1×n `Value::Tensor` of one-based indices; empty
  matrix → 1×0 empty row; a 1×1 result is an explicit 1×1 tensor so the output
  is uniformly a 1×n row vector (documented choice).
- **Permutation length `n`** = number of columns (equal to rows for the
  observed square inputs).
- **Non-square `dissect`** (unobserved) uses the column-intersection graph
  because `A+Aᵀ` is undefined for rectangular inputs — documented choice.
- **No MATLAB was invoked** in this repository (Principle II). Registration is
  the standard compile-time `#[runtime_builtin]` inventory macro; discovery via
  `builtin_function_by_name` is exercised by `descriptor_is_registered`.

## Unresolved (returned to specification side)

`colamd.q-permutation`, `colamd.q-values`, `dissect.q-permutation`,
`dissect.q-values` — the specific permutation values remain algorithm-dependent
and non-normative; wider-input orientation/permutation confirmation stays
spec-side.
