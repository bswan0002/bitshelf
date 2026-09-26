# Safety and recovery

Bits are ordinary UTF-8 Markdown. Keep backups of the store and configuration. No database or timestamp history is required to read, restore or copy a shelf.

Managed shelves, bits, configuration, guidance and lock paths refuse symlinks; the explicitly configured store root may itself be a link. Auxiliary files are preserved and not traversed. Identifiers are validated before path resolution.

Add and move never overwrite existing destinations. Edit compares the source bytes again before publishing a same-directory temporary file. Editors hold no writer lock while open; finalization locks and checks for conflicts. Failed drafts remain as hidden `.edit-*.md` or `.draft-*.md` files, with their path on stderr. Inspect both the original and draft before recovering content. Run `bs validate` after repair. Direct edits leave timestamp management to you.

Move publishes a destination before removing the source. A crash or partial failure can leave two copies; inspect both before retrying. Never assume an error means nothing was saved. Diagnostics identify publication and durability failures; successful cleanup is separate from content publication.

bs targets trusted local filesystems. Conflict checks are best effort against normal external editors, not filesystem compare-and-swap against hostile concurrent path replacement. No distributed/cloud-folder locking is promised. Atomicity avoids partial files; file and directory syncing additionally bound power-loss durability. There is no multi-file transaction. Attachments and links are not moved or rewritten.

Frontmatter comments and layout may normalize on mutation; supported values and body bytes are preserved. Read-only commands never stamp dates or create history. Obsolete prototype `.bitshelf/state.json` and `state.lock` files are ignored and left untouched, not migrated or removed.

Writers use an OS advisory lock at `.bitshelf/writer.lock`, waiting at most five seconds, then returning an actionable error. Closing the process (including SIGINT/SIGKILL) releases ownership automatically. The lock file remains; never delete it to resolve contention. Editors release ownership while waiting for the editor and acquire it only to commit. Read-only commands and dry runs do not create or acquire locks. Locking covers cooperating bs writers on supported local filesystems, not external editors or distributed storage.

Content/configuration commits write a temporary file in the destination directory, retain existing permissions (moves inherit source permissions), sync file data, publish atomically, then sync the parent directory. Supported release platforms are macOS and Linux. This bounds durability of entries in existing directories; newly created ancestor-directory chains and filesystem/device firmware remain outside a power-loss guarantee. Cross-filesystem moves sync the destination directory before unlinking the source, then sync its directory. If the latter fails, the destination is retained. Mode bits are preserved; inode identity, hardlink relationships, ACLs and extended attributes are not promised. Temporary files are cleaned up on ordinary pre-publication errors; abrupt termination may leave hidden temporary files for manual inspection.
