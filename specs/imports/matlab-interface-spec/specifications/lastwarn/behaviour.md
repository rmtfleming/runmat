# `lastwarn` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/lastwarn_observations.json`](observations/lastwarn_observations.json)

## Summary

Query or set the message and identifier of the most recent warning.

## Confirmed invocation forms

- `[msg, id] = lastwarn` — Last warning message and identifier. *(e.g. case `after-warning`)*
- `lastwarn(msg, id)` — Set the stored message and identifier. *(e.g. case `set`)*
- `[msg, id] = lastwarn(newMsg, newId)` — Set, returning the previous values. *(e.g. case `set-return`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — After warning('t:id', 'hello %d', 3), lastwarn returns the formatted message 'hello 3' and the identifier 't:id'; with one output it returns the message. *(cases: `after-warning`, `single-output`)*
2. **Confirmed** — lastwarn(msg, id) stores both values; lastwarn(msg) stores the message and an empty identifier; lastwarn('') clears the message and the identifier. *(cases: `set`, `set-msg-only`, `reset`)*
3. **Confirmed** — A setting call with outputs returns the previous message and identifier, then the new values are stored. *(cases: `set-return`)*
4. **Confirmed** — A warning whose identifier is turned off still updates lastwarn. *(cases: `disabled`)*
5. **Confirmed** — A warning raised without an identifier leaves an empty identifier. *(cases: `no-id`)*
6. **Confirmed** — After clearing, both outputs are 0x0 char. *(cases: `class`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `after-warning` | `struct` | `[1, 1]` | msg='hello 3'; id='t:id' |
| `single-output` | `char` | `[1, 6]` | 'second' |
| `set` | `struct` | `[1, 1]` | msg='m'; id='x:y' |
| `set-return` | `struct` | `[1, 1]` | msg='old'; id='o:ld'; after_msg='new'; after_id='n:ew' |
| `set-msg-only` | `struct` | `[1, 1]` | msg='only'; id='' |
| `reset` | `struct` | `[1, 1]` | msg=''; id='' |
| `disabled` | `struct` | `[1, 1]` | msg='hidden'; id='q:off' |
| `no-id` | `struct` | `[1, 1]` | msg='plain text'; id='' |
| `class` | `struct` | `[1, 1]` | msg_class='char'; msg_size=[0 0]; id_class='char'; id_size=[0 0] |

## Unresolved questions

- `lastwarn.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
