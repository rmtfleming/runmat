# Provenance Record — 001-clean-room-matlab-builtins

This is the programme-governance bootstrap feature. It imports **no**
behavioural specification and defines **no** MATLAB behaviour. Its provenance
record therefore documents the state of the specification source, not an
imported export.

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local path `/home/rfleming/drive/sbgCloud/code/matlab-interface-spec`; no remote recorded) |
| Approved specification identifier | none imported (governance feature) |
| Specification version / commit | **UNAVAILABLE** — source repository has no commits on `main` (blocker B-001) |
| Review status | n/a for this feature; candidate export `specifications/median/` carries per-claim reviewer attribution (ronan.mt.fleming@gmail.com) but no committed, citable approved revision |
| Import date | 2026-07-21 (date of this record; nothing imported) |
| Imported claim categories | none |

## Source-category discipline

When a built-in feature is bootstrapped under this programme, its
`provenance.md` MUST classify every relied-upon claim as exactly one of:

- `black-box-observation` — normative only if confirmed and reviewer-approved;
- `public-interface-fact` — normative;
- `independently-derived-inference` — NEVER normative; may motivate further
  specification-side work only.

## Blockers

- **B-001**: No citable committed revision of `matlab-interface-spec` exists.
  Resolution requires action in the specification repository (commit and mark
  an approved revision), which is out of scope for this repository and this
  task.
