# Deletion and expiry policies

Use only for requested deletion, pruning, or expiry-policy configuration. Inspect subcommand help first; installed 0.1.x candidates can have different features.

## Delete bits or an empty shelf

1. Resolve the user's intended IDs through discovery and inspect the material. Deletion is permanent, without a prompt or trash. Offer archiving when retention rather than destruction is intended.
2. Preview `bs delete <ID>... --dry-run --json`. For a shelf, preview `bs shelf delete <SHELF> --dry-run --json`; it only accepts an empty `bits/` plus optional `bs.toml` and `SHELF.md` (or a configured shelf whose bits directory is missing). Review the planned paths, including settings and guidance that will be lost.
3. Execute the same command without `--dry-run` only within the user's authorized scope. Shelf deletion refuses bits, scripts, attachments, symlinks, hidden files, and other unmanaged entries. There is no recursive flag; a refusal is not authorization to use recursive filesystem deletion instead.
4. Check both JSON and exit status. Bit deletion returns per-item rows; a missing/unsafe item may fail while others succeed. Shelf deletion returns one object with `planned` and `removed` paths; `partial` means some entries were removed before failure. After uncertain durability or partial shelf deletion, inspect the reported paths before retrying. Report completed and remaining work separately. Global aliases and other shelves' expiry destinations are not rewritten when a shelf is deleted.

## Configure archive on expiry

1. Load source and destination shelf contexts. Agree on retention, destination, and whether the destination should be permanent. Configuration alone does not authorize pruning existing material.
2. Preserve unrelated source `bs.toml` settings and add/update top-level keys outside any `[tag_rules.*]` table:

   ```toml
   retention = "14d"
   on_expire = { move = "archive/{shelf}.{name}" }
   ```

   Only `"delete"` (also the omitted default) or a built-in move object is accepted. `on_expire` requires retention. The destination must render to a full `SHELF/NAME` using only `{shelf}` and `{name}` placeholders. No aliases, shell commands, metadata options, or implicit destination creation. Changing this action applies to existing expired bits on the next prune.
3. Run `bs validate <SHELF> --json` for configuration/metadata checks and `bs prune <SHELF> --dry-run --json` for eligible-bit destination checks. Resolve collisions and destination requirements; a failed archive never falls back to deletion.
4. Confirm the policy via `bs context <SHELF> --json` and report preview outcomes. Expiry moves preserve all metadata, including `expires`, and add no `moved_from`. A permanent archive is never pruned; a retention-enabled destination can act on moved bits on the next invocation. Removing retention requires removing `on_expire` too.

## Execute requested pruning

Preview the exact scope with `bs prune [SHELF] --dry-run --json`, inspect removals and move destinations, then execute only the authorized scope. Omitting the shelf includes non-discoverable shelves too.

Prune returns per-item rows with source `id`/`path`, `status`, optional `destination`, and failure `error`. `would_remove`/`would_move` are previews; `removed`/`moved` are completed operations. Ordinary failures are `skipped` and allow other candidates to proceed. `removed_with_error`, `moved_with_error`, and `move_uncertain` stop the batch; inspect the source and any destination before retrying. Later candidates are untouched and absent from the array, so check the nonzero exit status rather than treating the array as an exhaustive success report.
