# Safety and recovery

Bits are ordinary UTF-8 Markdown. Keep backups of the store and configuration. No database or timestamp history is required to read, restore or copy a shelf.

## Paths and symlinks

Managed shelves, bits, configuration, guidance and lock paths refuse symlinks; the explicitly configured store root may itself be a link. At the store root, symlinked shelf directories are refused, while entries that aren't directories (including symlinks to files) are ignored. Auxiliary files are preserved and not traversed. Identifiers are validated before path resolution.

## Recovering a draft

Add and move never overwrite existing destinations. Edit compares the source bytes again before publishing a same-directory temporary file. Editors hold no writer lock while open; finalization locks and checks for conflicts. Failed drafts remain as hidden `.edit-*.md` or `.draft-*.md` files, with their path on stderr. Inspect both the original and draft before recovering content. Run `bs validate` after repair. Direct edits leave timestamp management to you.

## Interrupted moves

Move publishes a destination before removing the source. A crash or partial failure can leave two copies; inspect both before retrying. Never assume an error means nothing was saved. Diagnostics identify publication and durability failures; successful cleanup is separate from content publication.

## Deletion and expiration

[Explicit deletion](deleting.md) and default [pruning](pruning.md) permanently unlink bits without prompting or using the trash. Preview first and keep backups. Expiry policies can move bits instead; failed archival never falls back to deletion. Batch operations report partial outcomes and stop after uncertain publication, rollback, or durability. Inspect paths before retrying; an error may follow a completed removal or move.

## Scope and limits

bs targets trusted local filesystems. Conflict checks are best effort against normal external editors, not filesystem compare-and-swap against hostile concurrent path replacement. No distributed/cloud-folder locking is promised. Atomicity avoids partial files; file and directory syncing additionally bound power-loss durability. There is no multi-file transaction. Attachments and links are not moved or rewritten.

## Formatting and read-only commands

Frontmatter comments and layout may normalize on mutation; supported values and body bytes are preserved. Read-only commands never stamp dates or create history. Other files under `.bitshelf/`, such as `state.json` or `state.lock` left by pre-release builds, are ignored and left untouched.

## Writer locking

Writers use an OS advisory lock at `.bitshelf/writer.lock`, waiting at most five seconds, then returning an actionable error. Closing the process (including SIGINT/SIGKILL) releases ownership automatically. The lock file remains; never delete it to resolve contention. Editors release ownership while waiting for the editor and acquire it only to commit. Read-only commands and dry runs do not create or acquire locks. Locking covers cooperating bs writers on supported local filesystems, not external editors or distributed storage.

## Durable writes

Content/configuration commits write a temporary file in the destination directory, retain existing permissions (moves inherit source permissions), sync file data, publish atomically, then sync the parent directory. Supported release platforms are macOS and Linux. This bounds durability of entries in existing directories; newly created ancestor-directory chains and filesystem/device firmware remain outside a power-loss guarantee. Cross-filesystem moves sync the destination directory before unlinking the source, then sync its directory. If the latter fails, the destination is retained. New files use private `0600` mode, including new shell startup files created by completion installation; this is intentional rather than an umask-derived default. Existing mode bits are preserved; inode identity, hardlink relationships, ACLs and extended attributes are not promised. Temporary files are cleaned up on ordinary pre-publication errors; abrupt termination may leave hidden temporary files for manual inspection.
