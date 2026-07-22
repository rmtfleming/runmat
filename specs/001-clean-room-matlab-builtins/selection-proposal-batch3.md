# Selection Proposal (FR-010) — Batch 3: side-effect functions — 2026-07-22

Candidates: the six specifications added by the batch-2 import (packaged
export `runmat-export-2026-07-22`, commit pin `b054f3ad…`, all
`status: approved`). FR-002 eligibility: **fully satisfied (a)–(e)** — the
commit pin resolved B-001.

All six are absent from RunMat. All six have side effects; each feature
plan MUST include sandbox/test-isolation design at its gate (tests use
harmless fixed commands, temp files self-clean, no external network).

## Proposed feature grouping and assessment

| # | Feature (proposed) | Function | Substrate | Scope / risk | Isolation plan sketch |
|---|---|---|---|---|---|
| F11 | 011-system-invocation | `system` | `std::process` (used by plotting/unzip already) | Small–medium. Observed: status 0/1 passthrough, cmdout capture. Platform shell choice + wasm-unsupported error are documented choices. | Tests spawn only `true`/`false`/`echo`; no shell metacharacter tests; wasm returns a clear error. |
| F12 | 012-url-fetch | `urlread` | `io/http/transport.rs` (webread exists) | Small–medium. Observed case is a `file://` URL — transport must gain/route file-scheme reads. Legacy API kept thin. | Tests use `file://` temp files only; NO network in tests; http path reuses existing transport (already tested). |
| F13 | 013-display-formatting | `display` | `io/disp.rs`, console/output-context machinery | Medium. Observed outputs are exact text renderings (`    42\n\n`, right-aligned vectors, bare char). Must match RunMat's existing formatter or extend it; observed via `evalc` (RunMat needs equivalent capture for tests). | Pure formatting; capture output in-process; no fs/network. |
| F14 | 014-figure-export | `saveas` | plotting `export_figure_scene`, `print.rs` | Medium. Observed only file-created + nonempty. Adapter over existing export; plot-core feature gating; headless CI needs care. | Temp-file outputs, self-cleaning; skip gracefully when plot-core disabled. |
| F15 | 015-input-parsing | `inputParser` | `ObjectInstance`/handle-object machinery ("until full class system lands") | **Large / highest risk.** Stateful handle-like object with methods (addRequired, addParameter, parse) and properties (Results struct, UsingDefaults cell). Object-system maturity must be assessed in the plan; may surface a prerequisite feature. | No external side effects; risk is architectural, not sandbox. |
| — | (blocked) | `writetable` | **No table type (B-010-1)**; roundtrip observation also needs `readtable` (absent) | BLOCKED — joins feature 010. Unblocks when a table type + readtable exist. | n/a |

## Proposed order

F11 → F12 → F13 → F14 → F15. Rationale: ascending risk; F11/F12 are thin
adapters over existing substrate; F13 touches shared formatting; F14 depends
on plot-core; F15 may need an object-system prerequisite decision.

## Decision — RECORDED 2026-07-22

- [x] Grouping and order **APPROVED** as proposed (maintainer, interactive
  gate): F11 → F12 → F13 → F14 → F15.
- [x] **All five features authorised for B1** under the established
  batch-review cadence.
- [x] `writetable` blocked status acknowledged — tracked with feature 010
  until a table type (and `readtable`) exist.
