# Deleting bits

`bs delete <ID>...` permanently removes explicitly named bits. It does not prompt or use the trash. Preview first:

```sh
bs delete notes/obsolete snippets/unused --dry-run --json
bs delete notes/obsolete snippets/unused --json
```

Use [move or archive](moving.md) instead when you may want the material later. Keep backups for recovery. Shelf deletion is limited to empty shelves (below); there is no recursive-delete command.

- IDs must use exact stored spelling. There are no globs or implicit shelf-wide deletion; duplicate IDs are processed once, in first-occurrence order.
- Only regular bit files are removed. Symlinks, directories, and other special files are refused. Unrelated shelf files and guidance stay untouched.
- Explicit deletion can remove a bit with malformed metadata or non-UTF-8 contents; expiration and shelf requirements are not deletion prerequisites.
- Actual deletion holds the store writer lock, checks original bytes before unlinking, and syncs the parent directory. Dry runs check the targets without acquiring a lock or changing files; they are previews, not reservations.
- Missing IDs and other per-item failures produce `skipped` rows with errors; other requested bits are still processed. This is not an all-or-nothing transaction.
- A post-removal durability failure produces `removed_with_error` and stops the batch. Later IDs are untouched and absent from the results. Inspect the reported path before retrying.

JSON output is a per-item array with `id`, `path` (null if resolution failed), `status`, and optional `error`. Status is `would_remove`, `removed`, `skipped`, or `removed_with_error`. Any failure exits 1 after reporting completed work. Missing operands and unknown flags exit 2. See [Safety and recovery](safety.md).

## Deleting an empty shelf

```sh
bs shelf delete scratch --dry-run --json
bs shelf delete scratch --json
```

`bs shelf delete <SHELF>` removes an empty `bits/` directory, optional `SHELF.md` and `bs.toml`, and then the shelf directory. Settings and guidance are permanently deleted without prompting. A configured shelf missing its bits directory is also accepted; an arbitrary directory without `bits/` or `bs.toml` is not a shelf.

Bits (including hidden drafts), scripts, attachments, symlinks, and any other entries cause refusal **before deletion starts**. Move or delete bits explicitly first, and handle unmanaged material separately. No `--recursive` or force option is available. Global configuration, aliases, and other shelves' expiry destinations are not rewritten; review references to the removed shelf yourself.

Shelf deletion holds the writer lock and rechecks the remaining contents before every removal. This is not an atomic directory transaction: an external edit or filesystem failure can stop it after some entries were removed. It never recursively removes newly appearing contents.

JSON is one object with `name`, `path`, `status`, `planned` paths, `removed` paths, and optional `error`. `would_remove` previews the plan; `removed` means the plan completed. `skipped` means nothing was removed; `partial` means some entries were removed before failure (including possible durability failure). Failures exit 1. Inspect remaining files before retrying; an interrupted shelf with neither `bits/` nor `bs.toml` may need manual cleanup. Restore settings/guidance from backups if needed.
