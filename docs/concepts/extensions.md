# Aliases and extensions

Extend `bs` without adding built-ins. Invocation resolves **built-in → configured alias → `bs-NAME` executable on PATH**. Within PATH, the first executable wins. Configured names cannot shadow built-ins. Names use lowercase ASCII letters, digits, and hyphens, and cannot start with a hyphen.

Aliases live in the global configuration only. Shelves and repository files never register commands or trigger execution. Extensions run only when invoked; treat their definitions and executables as trusted code. Their presence is not permission to run them for an unrelated task.

## Discover and inspect

```sh
bs --help                                  # built-ins plus available extensions
bs aliases                                 # name, kind, description
bs aliases --json                          # structured catalog keyed by name
bs aliases show last --json                # definition, interface, origin
bs aliases dry-run last --json -- notes --count=20
```

Listing, showing, and previewing **never execute an extension**, including its `--help`. They work before store initialization. Missing config means no configured aliases; PATH discovery still works. Unreadable or invalid existing config is an error for inspection/invocation; top-level help warns and still displays built-in help and PATH extensions.

`dry-run` renders the exact executable, argv, environment overrides, and (for recipes) shell command. Put preview arguments after `--`. The inspector's own `--json` controls preview output; put a target `--json` after the separator. This previews invocation, not the program's internal behavior or side effects. It does not check that a move destination exists, for example. A target's own `--dry-run` is separate and may execute code.

`show` includes descriptions, usage, examples, implementation (`argv`, `exec`, or `run`), origin, and recipe parameters/defaults. Usage and examples are documentation, not an argument parser or a safety guarantee. Missing descriptions are explicit. See the [JSON contract](../json.md).

## Built-in argv shortcuts

```toml
[aliases]
recent = ["list", "--sort", "created", "--reverse"]
archive = ["move", "{id}", "archive/{shelf}.{name}", "--set", "moved_from={id}"]
```

Plain shortcuts append arguments: `bs recent notes --json`. Arrays must begin with a built-in command; they cannot chain to aliases. They use no shell, environment expansion, or tilde expansion.

ID shortcuts accept exactly one `shelf/name` ID. `{id}`, `{shelf}`, and `{name}` substitute once, preserving argument boundaries. Templates bind to the built-in grammar before substitution, so IDs that look like flags remain operands. Unknown or unclosed placeholders are configuration errors.

```sh
bs archive notes/checklist --dry-run --json
bs archive --dry-run -- -notes/checklist
```

ID shortcuts accept only `--json`, `--dry-run`, and `--config`; configure other arguments in the array. The target must support `--dry-run`. Built-in shortcuts preserve the target's output and exit status. `bs recent --help` safely shows its definition.

Use a table to document a shortcut:

```toml
[aliases.recent]
argv = ["list", "--sort", "created", "--reverse"]
description = "List bits newest-created first"
usage = "recent [SHELF] [LIST_OPTIONS...]"
examples = ["bs recent notes", "bs recent notes --json"]
```

Every alias table requires exactly one of `argv`, `exec`, or `run`. `description`, `usage`, and `examples` are optional for all three. Unknown fields are rejected.

## Shell recipes

Strings are shell recipes. Tables add documentation and parameter defaults:

```toml
[aliases]
hello = "printf 'hello\\n'"

[aliases.last]
description = "List the newest bits by creation time"
usage = "last <shelf> [--count=N]"
examples = ["bs last notes", "bs last notes --count=20"]
run = "bs list {{ args }} --sort created --reverse | awk -v n={{ count }} 'NR <= n'"
defaults = { count = "10" }
```

```sh
bs last notes
bs last notes --count=20
```

The template language deliberately supports only `{{ variable }}`:

| Variable | Value |
| --- | --- |
| `args` | Remaining arguments, individually shell-escaped and space-joined |
| `store_path` | Resolved store directory |
| `config_path` | Absolute active configuration path |
| `bs_executable` | Current executable path |
| Any other name | A named parameter supplied as `--name=VALUE`, or its configured default |

Variable names use ASCII letters, digits, and underscores and cannot begin with a digit. `config` is unavailable as a parameter because `--config` selects bs configuration; use `config_path` for context. Parameters are discovered from the template. Defaults must be strings and may only name referenced, non-context parameters. Missing required values fail before any shell execution. Repeated bindings use the last value. The engine does not validate parameter types: a numeric count is the recipe's responsibility.

`--KEY=VALUE` binds when KEY is a referenced user parameter; otherwise it forwards into `args`. `--KEY VALUE` is not binding syntax. `--` is removed and all following tokens forward literally, bypassing binding. Context variables cannot be overridden by arguments. There is no implicit current shelf.

**Place placeholders unquoted in shell argument positions.** Each value is already shell-escaped, including paths. For example, use `printf '%s\n' {{ label }}`, not `printf '%s\n' "{{ label }}"`. Trusted recipe authors must not place substitutions in shell quotes, comments, heredocs, arithmetic, or other code contexts, or re-evaluate supplied values with `eval`/`sh -c`. Escaping preserves data arguments; it is not a sandbox for arbitrary shell programs. Templates are rendered once; there are no filters, indexing, loops, raw blocks, or multi-step pipeline tables.

### Shell and process contract

Recipes use `/bin/bash --noprofile --norc -e -o pipefail -c`. `BASH_ENV` is removed for recipe execution. Bash's normal `errexit` rules apply; `pipefail` exposes upstream failures, including SIGPIPE. The example uses `awk` to consume the whole stream rather than `head`, which can close a pipe early. Recipe authors must handle expected failures explicitly.

Recipes inherit the caller's working directory and standard streams. Shell changes such as `cd` or `export` do not change the parent shell. A shell function named `bs` routes nested calls through `BS_EXECUTABLE`; `BS_CONFIG` preserves the selected config. The function appears in rendered previews. Explicit `command bs` or a separately invoked program can bypass that function.

`bs RECIPE --help` shows the definition without executing it. To forward a literal help flag, use `bs RECIPE -- --help`. Other flags, including `--json` and `--dry-run`, have no built-in recipe semantics unless bound/forwarded by the recipe. Recipes own their output contract and side effects. Avoid recursive recipes.

## Explicit helpers

Register an executable under any command name, with optional fixed arguments:

```toml
[aliases.tickets]
exec = ["python3", "/absolute/path/to/tickets.py", "--team", "platform"]
description = "Import team tickets into the configured shelf"
usage = "tickets ISSUE_KEY"
examples = ["bs tickets TEAM-123"]
```

The executable and fixed arguments are literal argv. No shell or placeholder expansion occurs. Program names resolve through PATH; relative paths use the caller's working directory. Prefer absolute script paths. Compact `{ exec = ["program"] }` syntax also works.

## Automatic executable discovery

Install an executable named `bs-sync` on PATH and invoke it as `bs sync`. No alias registration is necessary. Arguments pass directly to the executable; it can be written in any language. Non-executable files, invalid extension names, and built-in names are ignored. Configured aliases take precedence. Discovery does not run programs.

Optionally place a metadata file **beside the PATH entry**, named `bs-sync.toml`:

```toml
description = "Synchronize the store with a remote"
usage = "sync [--dry-run]"
examples = ["bs sync --dry-run"]
```

This sidecar contains documentation only, not executable definitions or configuration. Unknown fields or unreadable/invalid sidecars produce a `metadata_error` on the catalog entry; the executable remains discoverable. With no sidecar, description and usage are null. Symlinked executables look for the sidecar beside the symlink, not its target.

### Helper and executable process contract

- Arguments, including `--help`, `--json`, and the literal `--` separator, forward unchanged, except `--config PATH` / `--config=PATH` before the separator: these select the bs configuration and are consumed. A preceding global `--json` is forwarded too.
- `BS_CONFIG` contains the absolute selected configuration path. Config resolution is explicit `--config` → `BS_CONFIG` → XDG default.
- `BS_EXECUTABLE` contains the running bs executable path. Helpers should call `"$BS_EXECUTABLE"` rather than assume another `bs` on PATH is the same version.
- Working directory and standard streams are inherited. Helpers own help, JSON support, side effects, and output. Exit codes propagate; signal termination maps to 1.
- `bs HELPER --help` **executes the helper**. Use `bs aliases show HELPER` for safe inspection instead.

There are no automatic lifecycle hooks, repo-config approvals, or sandboxing. Dynamic extension-name/argument completion is not implemented; built-in completion continues to work. For larger workflows, use an external script rather than growing shell templates into an orchestration system.
