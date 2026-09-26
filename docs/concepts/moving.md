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

Aliases live only in the global configuration, not in shelves. Built-in aliases are argument arrays starting with a built-in command:

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
- Built-in aliases expand arguments only; there is no implicit shell or environment expansion.
- Alias names and their arguments aren't dynamically completed yet; built-in completion still works.

### Explicit helper aliases

Opt into an external helper with an `exec` argument array in your **global** configuration:

```toml
[aliases]
ticket-list = { exec = ["python3", "/absolute/path/to/shelf/scripts/ticket-list.py"] }
```

This runs only when you explicitly invoke `bs ticket-list`. Treat configured executables and scripts as trusted code. Shelf configuration and guidance cannot register executable aliases automatically.

- The executable and fixed arguments are literal argv; there is no implicit shell, placeholder substitution, tilde expansion, or environment expansion. Executable names resolve through `PATH`; relative paths use the caller's working directory. Prefer absolute script paths.
- Arguments after the alias, including `--help` and `--json`, go to the helper unchanged, except `--config PATH` / `--config=PATH`, which select the bitshelf configuration and are consumed. A preceding global `--json` is forwarded too. Arguments after `--` remain literal.
- `BS_CONFIG` contains the absolute selected configuration path. `BS_EXECUTABLE` contains the current bs executable path. Helpers calling bs should use both, so configuration overrides and the running version remain consistent.
- Helpers inherit the working directory and standard streams. They own help, output, JSON support, and side effects; bs does not turn arbitrary helper output into JSON. Exit codes are preserved (signal termination maps to 1).
- `bs aliases --json` exposes helper aliases as `{ "exec": [...] }`; built-in aliases remain arrays. Inspecting aliases does not execute them.

This is explicit command execution, not a hook or plugin lifecycle. A helper may call built-in bs commands; avoid configuring recursive helper invocations.

See [Archive without deleting](../recipes/archive.md) for the archive alias in context.
