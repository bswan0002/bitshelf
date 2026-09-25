# Timestamps and sync

Every bit has `created` and `updated` timestamps that `bs` maintains for you, so sorting by recency stays trustworthy even when files are edited outside `bs`.

## The short version

- **Never set `created` or `updated` yourself.** There are no flags for them, and manual changes to tracked values are restored.
- Changes made through `bs add`, `bs edit`, and `bs move` are timestamped automatically.
- After editing files directly (in your editor, with `bs open`, or with a script), run `bs sync`.
- When you start using existing Markdown files, run `bs sync` **before** editing them externally.

```sh
bs sync --dry-run      # preview reconciliation
bs sync                # reconcile all shelves
bs sync notes --json   # reconcile one shelf
```

## How it works

**Reserved fields.** `created` is fixed once tracking begins. `updated` advances automatically when the body or user metadata really changes. A no-op edit of a tracked, reconciled bit leaves timestamps unchanged. First-time tracking can fill in missing dates, and an edit can pick up a pending external change. Frontmatter key order, comments, and changes to reserved timestamps alone aren't content changes. Generated dates use UTC; valid imported dates keep their representation. Rewriting timestamps may normalize YAML formatting and comments, but the Markdown body is preserved exactly.

**Change detection.** `bs sync` compares SHA-256 hashes of each bit's body and non-reserved metadata against hidden bookkeeping in `<store>/.bitshelf/state.json`. `updated` records the **time the change was detected**, not a file modification time. There is no background watcher, and `list`, `show`, `search`, and `validate` never reconcile.

**Existing and imported files.** The first sync establishes a baseline. Valid existing timestamps are kept, and missing ones are set to the time the file is first tracked. Historical dates are never invented, and `created` is never taken from the file's birth time. Edits made before the baseline can't be detected, which is why you should sync imported files before editing them externally. A previously unused identifier is baselined when discovered.

**Identity.** State is keyed by identifier, not inode, so editors that save by replacing the file keep their history. `bs add` always starts fresh history for its new bit. Moves carry history to the new ID. Add, edit, move, sync, and prune discard records for paths observed missing, including deleted shelves, and pruning also discards records for removed bits. Run sync after deleting or renaming files, before reusing their old identifiers. A file deleted and replaced at the same identifier between commands is indistinguishable from an edit and keeps that identifier's history.

**Losing state.** Losing `state.json` loses change history, not notes or frontmatter dates; the next sync re-baselines. Include it in backups if you want to keep change detection, and don't edit it by hand.

## What sync does and doesn't do

- It reconciles readable bits even when shelf requirements (title, tags) are missing or user metadata is invalid. Use `bs validate` for those; sync never fills them in.
- Malformed YAML, invalid untracked timestamps, and read/write failures produce per-bit errors. Other bits are still processed, and sync exits nonzero if any bit failed.
- `--dry-run` writes neither bits nor state.
- It preserves expiration and unknown metadata, never extends retention, never deletes notes, and never prunes.

For locking and interrupted writes, see [Safety and recovery](safety.md).
