# Moving bits

```sh
bs move notes/checklist projects                    # keep the name; shelf must exist
bs move projects/checklist notes/release-checklist  # move and rename
bs move notes/release-checklist projects --set status=done --dry-run --json
```

The destination is a shelf (keeping the bit name) or a complete `shelf/name` ID.

- Moves refuse existing destinations, identical source and destination IDs, unsafe paths, and invalid destination metadata.
- The **destination** shelf's requirements and tag rules apply; the source's don't. Read the destination's `bs context` guidance before moving content there.
- `--set <KEY=VALUE>` (repeatable) sets a frontmatter **string**, preserving other metadata and the exact body. Values may contain `=`. It isn't YAML evaluation: `--set tags=…` can't build a list. `created` and `updated` stay reserved. Repeating a key, or combining conflicting operations on the same field, fails before anything is written. See [Metadata mutations](metadata.md) for `--set-json`, `--unset`, and the full rules.
- `--dry-run` validates and returns the planned destination without writing files.

**Timestamps.** A plain move preserves dates including absence. Metadata changes update `updated` relative to the current source; `created` is never invented. External edits already in the source are authoritative.

**Expiration.** `expires` is preserved: never added, extended, or removed by a move. A bit moved into a retention shelf may therefore already be eligible for pruning, and one without `expires` is reported as skipped by prune. Permanent shelves are never pruned, even when bits have expiration metadata.

**IDs change.** The old ID stops resolving, and references to it elsewhere aren't rewritten.

For how moves are written and recovered after interruption, see [Safety and recovery](safety.md#interrupted-moves).

## Archive workflows

Archive is a configured workflow, not a built-in command. Follow
[Archive without deleting](../recipes/archive.md) to set up an archive shelf and
an alias that moves bits there. See [Extending bitshelf](extensions.md) to learn
how command shortcuts, shell recipes, and external helpers work.

## References and auxiliary files

Moving changes the path-derived ID; the old ID stops resolving. Bodies are byte-preserved, so links to the old ID/path and relative links inside the moved body may break. Attachments, scripts, images and other auxiliary files are not moved or rewritten. Review such references yourself. Only the final destination candidate must satisfy destination metadata rules; publication never overwrites another file.

Moving into a retention-enabled shelf does not add an expiration. A missing expires remains conservatively unprunable; an already-expired explicit expires becomes eligible immediately. Edits do not extend expiration. Permanent shelves are never pruned, even with past expiration dates.
