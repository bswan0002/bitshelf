# Moving and aliases

## Moving

```sh
bs move notes/checklist projects                    # keep the name; shelf must exist
bs move projects/checklist notes/release-checklist  # move and rename
bs move notes/release-checklist projects --set status=done --dry-run --json
```

The destination is a shelf (keeping the bit name) or a complete `shelf/name` ID.

- Moves refuse existing destinations, identical source and destination IDs, unsafe paths, and invalid destination metadata.
- The **destination** shelf's requirements and tag rules apply; the source's don't. Read the destination's `bs context` guidance before moving content there.
- `--set KEY=VALUE` (repeatable) sets a frontmatter **string**, preserving other metadata and the exact body. Values may contain `=`. It isn't YAML evaluation: `--set tags=…` can't build a list. `created` and `updated` stay reserved. For duplicate keys, the last assignment wins.
- `--dry-run` validates and returns the planned destination without writing files or state.

**Timestamps.** Moving alone doesn't advance `updated` for a reconciled bit; metadata changes or pending external edits do. `created` and tracking history follow the bit to its new ID. Untracked files are baselined first, missing timestamps may be filled, and frontmatter formatting may be normalized when metadata is rewritten.

**Expiration.** `expires` is preserved: never added, extended, or removed by a move. A bit moved into a retention shelf may therefore already be eligible for pruning, and one without `expires` is reported as skipped by prune. Permanent shelves are never pruned, even when bits have expiration metadata.

**IDs change.** The old ID stops resolving, and references to it elsewhere aren't rewritten.

For how moves are written and recovered after interruption, see [Safety and recovery](safety.md#interrupted-moves).

## Aliases

Aliases are shortcuts for built-in commands. They live only in the global configuration, not in shelves. Each is an argument array starting with a built-in command:

```toml
[aliases]
recent = ["list", "--sort", "updated", "--reverse"]
archive = ["move", "{id}", "archive/{shelf}.{name}", "--set", "moved_from={id}"]
```

**Plain aliases** append your arguments unchanged: `bs recent notes --json`.

**ID aliases** use placeholders and accept exactly one bit ID: `{id}` is the complete ID, `{shelf}` its shelf, and `{name}` its name. `bs archive notes/checklist` expands to `move notes/checklist archive/notes.checklist --set moved_from=notes/checklist`. Substitution is single-pass and keeps each argument intact, including spaces and shell metacharacters. ID aliases accept `--json`, `--dry-run`, and `--config` before or after the ID; put any other arguments in the array. `--dry-run` must be supported by the target command.

```sh
bs aliases --json        # name → argv map
bs archive --help        # shows the expansion
```

- Global `--config` and `--json` also work before the alias name.
- Aliases return their target command's JSON result and exit status.
- Aliases can't shadow built-ins or chain to other aliases. Names use lowercase ASCII letters, digits, and hyphens, and can't start with a hyphen. Unknown or unclosed placeholders are configuration errors.
- Aliases expand arguments only. There are no shell commands, hooks, pipelines, or environment expansion. For multi-step workflows, write an ordinary script that calls `bs`.
- Alias names and their arguments aren't dynamically completed yet; built-in completion still works.

See [Archive without deleting](../recipes/archive.md) for the archive alias in context.
