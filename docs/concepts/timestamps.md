# Timestamps

> Approved target contract (ticket 01). The prototype still uses lifecycle state
> and `bs sync`; implementation follows in ticket 09. The rules below describe
> the replacement, not the current executable.

## Authority and scope

Markdown is authoritative. There is no persistent timestamp history, baseline,
watcher, or reconciliation. `bs` compares only the original and proposed content
within one command. Direct edits, including edits after `bs open`, leave timestamp
management to the user. Read commands and validation never rewrite dates.

`created` and `updated`, when present, must be RFC 3339 strings with an explicit
offset. Missing fields are permitted unless shelf requirements demand them; null,
empty strings, date-only strings, and non-string values are invalid. Valid imported
values retain their string representation unless the command replaces them.
Generated values use UTC (`Z`) with sufficient fractional precision to avoid
rounding below a preserved timestamp. No filesystem time supplies either date.
There is no requirement that `created <= updated`: imported or explicitly edited
dates may express user-managed history.

## Command rules

| Operation | `created` | `updated` |
| --- | --- | --- |
| `add`, including interactive add | Set to successful-finalization time | Set to the same instant as `created` |
| `edit` with a body/user-metadata change | Preserve original value or absence unless explicitly edited in the draft | Automatically stamp unless explicitly edited in the draft |
| No-op `edit` | Preserve | Preserve |
| Plain `move` (ID, shelf, or path changes only) | Preserve, including absence | Preserve, including absence |
| `move` that changes body/user metadata | Preserve, including absence | Automatically stamp |
| `open`, external editing, discovery, read commands, validation | No automatic changes | No automatic changes |

For add, sample one clock instant after editor completion and validation, near
publication, not when the draft is opened. Add owns both dates: draft values or
removals are overwritten with that instant, not imported as historical dates.
Malformed frontmatter still fails. Retention policy is separate; this contract
neither extends expiration nor changes how expiration is chosen.

For automatic updates, use `max(finalization time, original updated)` if an
original `updated` exists, otherwise finalization time. If the prior instant wins,
preserve its representation. A changed bit need not get a strictly greater date:
backward-moving or equal clocks must not fabricate a future increment or prevent
the save. A missing `created` is never invented by edit or move. Missing dates on
no-op edits and plain moves remain missing.

## What counts as a change

Compare the exact UTF-8 body bytes and the supported parsed user-metadata values,
excluding only top-level `created` and `updated`. `expires` and unknown user fields
participate. Mapping order, comments, quoting style, and equivalent YAML syntax
do not constitute a semantic metadata change. Sequence order, field absence vs.
null, value types, body whitespace, line endings, and final newlines do matter.
Nested mapping order is likewise irrelevant. Ticket 02 defines the supported
value model; comparison must not use lossy JSON conversion.

Compare against the original read by this command, never an older version.
An external edit that predates the command is already authoritative and is not a
new change to detect. A move's new identifier is not user metadata.

A semantic no-op leaves dates unchanged. An identical proposed file requires no
rewrite. Explicit formatting/comment-only draft edits may be published without
advancing dates; syntax preservation on other rewrites is governed by the
frontmatter contract.

## Explicit editor-draft dates

For editor-mode edit, compare each draft date field with the original field by
presence and parsed string value. A different valid string (even for the same
instant) or removal is an explicit edit. Quoting-only changes to the same string
are not. An unchanged final draft cannot reveal intermediate editor actions.

- Honor explicit valid `created` changes or removal.
- Honor explicit valid `updated` changes or removal, even alongside content
  changes. This overrides automatic stamping and the clock clamp for that field.
- Changing only `created` does not trigger an automatic `updated` change.
- Date-only edits are saved but are not body/user-metadata changes.
- Removals must still satisfy shelf-required fields.

Invalid existing dates are not silently repaired by automatic stamping. An edit
draft can explicitly replace an invalid date with a valid value or remove it;
otherwise edit/move fails validation, including on no-ops and plain moves. A
body-only or unrelated metadata mutation cannot hide an invalid original date.
Malformed YAML must be repaired outside the command if a draft cannot be parsed.
No new timestamp mutation flags are introduced here; the shared metadata contract
must preserve these automatic-field rules.

## Finalization and failure

Editor-mode edit waits for successful editor exit, compares the original with the
final draft, validates, and checks that the source has not changed concurrently.
Only final publication stamps the bit; intermediate editor saves never stamp the
source. Nonzero editor exit, invalid drafts, destination collisions, concurrency
conflicts, and pre-publication write failures leave source bytes and dates alone.
Failed editor drafts remain available for recovery.

A publication followed by a durability or cleanup failure may already have saved
the new dates. Report that outcome accurately rather than claiming rollback or
retrying blindly. Move partial-publication outcomes follow the filesystem commit
contract; timestamps do not imply transactionality. No separate timestamp-state
commit exists.

## Removing prototype history

Remove `bs sync` rather than retain an alias or no-op compatibility command.
Stop reading, writing, validating, or repairing `.bitshelf/state.json`. Ignore
existing prototype history, even if corrupt; leave it on disk for optional manual
removal. Do not migrate it or restore dates from it. Identifier reuse has no history.

Writer locking remains independent of timestamp authority. The lock implementation
must not depend on the prototype `state.lock` sentinel; obsolete state/lock files
must not block commands. Do not recursively delete `.bitshelf`, which may contain
independent locking or other files. Ticket 10 defines the replacement locking.

See [Saving and editing](editing.md) and the
[timestamp fixture specification](timestamp-fixtures.md).
