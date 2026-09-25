# Safety and recovery

bitshelf is a prototype. Keep backups (Git works well for a store) and use disposable data. This page collects what `bs` guarantees, where it stops, and how to recover when something goes wrong.

## What `bs` guarantees

- **No overwriting on create or move.** New bits and moved bits are published with no-clobber temporary files; collisions fail.
- **Safe edits.** Edits atomically replace the original after checking it didn't change concurrently.
- **Exact bodies.** Bodies are preserved byte-for-byte; only frontmatter is ever rewritten.
- **Contained paths.** Paths can't escape a shelf through traversal.
- **No symlinks in managed paths.** Shelves, `bits/`, bit files, `bs.toml`, `SHELF.md`, and tracking state refuse symlinks, including links that point inside the store. The explicitly configured store root may be a symlink. Discovery warns and skips linked shelves and bits, and validation reports links, including dangling links and non-hidden links at the store root, whose targets aren't inspected. Auxiliary files such as `scripts/` aren't traversed.
- **Serialized lifecycle writes.** Add, edit, move, sync, and prune take `.bitshelf/state.lock` to prevent competing timestamp writes and pruning. An edit holds no lock while the editor is open. At finalization it takes the lock, reloads state, and refuses to overwrite an original that changed in the meantime.

## Where it stops

- Updating a bit and updating tracking state are separate atomic writes, not one transaction.
- A move across files isn't a transaction either; a crash can leave both copies.
- Treat the store as a trusted local directory. Hostile concurrent filesystem replacement, a store writable by untrusted users, and concurrent configuration writers are outside the prototype's guarantees.
- Pruning deletes files permanently.

## Recovery

### "The bit was saved" but a command failed

An error that explicitly says the bit **was saved** means only bookkeeping failed. Fix the reported problem and run `bs sync`. **Don't repeat `bs add`**, which would fail on the collision or create a duplicate under another name.

### Recovering a draft

If interactive add or editor-mode edit fails, the draft's path (`.draft-*.md` or `.edit-*.md` in the shelf's `bits/`) is printed on stderr. First inspect the destination. If it wasn't saved, resolve the failure (such as missing required metadata), then recover the draft's content without overwriting a newer bit, and run `bs sync` and `bs validate`.

### Interrupted moves

A move holds the lifecycle lock, writes a destination-local temporary file with the source's permissions, publishes it without clobbering, checks the source for concurrent edits, and only then removes the source. It works across filesystems. If removing the source fails, `bs` tries to roll back the destination and reports any rollback failure.

- If diagnostics say the move completed but bookkeeping failed, run `bs sync`. Don't repeat the move.
- If a crash left both copies, inspect both paths, keep the intended copy, and run `bs sync`.

### Interrupted writes

After an interrupted add, edit, or sync, rerun `bs sync`. A pending edit may be timestamped at the time of the retry.

### Stale lock

After a crash, `.bitshelf/state.lock` may remain. Remove it only after confirming no `bs` command is running.

### Lost tracking state

If `.bitshelf/state.json` is lost, notes and frontmatter dates are intact; only change history is gone. The next `bs sync` re-baselines. See [Timestamps and sync](timestamps.md).

## Formatting normalization

- Saving shelf configuration normalizes TOML formatting and comments.
- `bs edit`, `bs move`, and `bs sync` may normalize YAML frontmatter formatting and comments. Bodies are never changed.
