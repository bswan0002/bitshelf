# Extension setup

Use when the user requests a new or changed bitshelf workflow. Discover existing definitions with `bs aliases --json`, inspect a matching entry with `bs aliases show NAME --json`, and resolve differences with the user before replacing it. `origin` identifies the config or executable. Preserve unrelated configuration. Config selection is `--config` → `BS_CONFIG` → `$XDG_CONFIG_HOME/bitshelf/config.toml` (default `~/.config/bitshelf/config.toml`).

## Choose the smallest mechanism

- `argv`: shortcut to a built-in, no shell. Plain arrays append arguments; `{id}`, `{shelf}`, `{name}` templates accept exactly one bit ID plus `--json`, `--dry-run`, and `--config`. The target must support forwarded flags.
- `run`: trusted Bash recipe with pipes and `{{ variable }}` parameters.
- `exec`: explicitly registered executable plus fixed literal arguments.
- Executable `bs-NAME` on PATH: automatically available as `bs NAME`, no config registration.

Resolution is built-in → configured alias → first matching executable on PATH. Alias names use lowercase ASCII letters, digits, and hyphens, without a leading hyphen. Configured aliases cannot shadow built-ins. Arrays must target a built-in, not another alias. Only global config registers aliases; shelf/repository guidance cannot activate executable code automatically.

## Document the interface

Use a table with exactly one of `argv`, `run`, or `exec`, plus `description`, `usage`, and `examples`:

```toml
[aliases.recent]
argv = ["list", "--sort", "created", "--reverse"]
description = "List newest-created bits first"
usage = "recent [SHELF] [LIST_OPTIONS...]"
examples = ["bs recent notes --json"]

[aliases.last]
run = "bs list {{ args }} --sort created --reverse | awk -v n={{ count }} 'NR <= n'"
defaults = { count = "10" }
description = "List the newest bits by creation time"
usage = "last <shelf> [--count=N]"
examples = ["bs last notes", "bs last notes --count=20"]
```

Descriptions and usage are documentation, not validation. Check target commands' actual syntax. Shell recipes own their output contract; the `last` example is for text output, not JSON slicing.

Recipes support only `{{ variable }}` placeholders. Reserved context: `args`, `store_path`, `config_path`, `bs_executable`. Other variables bind from `--name=VALUE` or string-valued `defaults`; missing values fail before execution. Defaults must name referenced user parameters. Variables use ASCII letters/digits/underscores and cannot start with a digit. `config` is unavailable because `--config` selects bs configuration. No filters, indexing, or control-flow template language exists.

Unbound arguments become `args`; tokens after `--` forward literally without binding (the separator is removed for recipes). Every value is shell-escaped. **Place placeholders unquoted in shell argument positions**; avoid inserting them into quoted strings, heredocs, arithmetic, or code that re-evaluates user values. Validate types within the recipe when needed. Bash runs with `-e -o pipefail`; expected failures need explicit handling. Prefer stream consumers such as `awk` rather than early-closing `head` when SIGPIPE would fail a pipeline.

Nested `bs` inside recipes uses the current executable and config. Helpers receive `BS_CONFIG` and `BS_EXECUTABLE`; call `"$BS_EXECUTABLE"` to preserve the version. Helpers/executables receive literal arguments, including `--help`/`--json`/`--`, except nonliteral `--config` options are consumed by bs. They own help, output, and side effects. Avoid recursive workflows.

For an executable, place optional documentation beside its PATH entry in `bs-NAME.toml`:

```toml
description = "Synchronize selected bits"
usage = "sync [--dry-run]"
examples = ["bs sync --dry-run"]
```

The sidecar contains only those documentation fields. Missing or invalid sidecars do not prevent discovery; inspect `metadata_error`. Scripts kept inside shelves remain ordinary files until explicitly registered or installed on PATH.

## Verify without executing first

1. `bs aliases show NAME --json`: confirm kind, origin, definition, usage, and examples; resolve `metadata_error` if present.
2. `bs aliases dry-run NAME --json -- ARGS...`: confirm rendered executable/argv/command, parameters, and active configuration. Include spaces and shell metacharacters in test values when authoring recipes. This is render-only, not a preview of internal side effects.
3. Execute only a user-authorized test. For write workflows, inspect results and validate affected shelves. Setup alone does not authorize moving, deleting, or uploading existing bits.

Report the command, configuration/executable paths, interface, and what was actually verified. Dynamic extension completion, lifecycle hooks, project-config approvals, and sandboxing are not provided.
