# JSON contract (0.1.x)

`--json` is global. Successful structured output is one document on stdout; diagnostics go to stderr. Arrays are used for collection results, including `[]`. Paths are absolute. Nullable fields are emitted as `null`, not omitted. Consumers should tolerate new fields.

| Command | Result |
| --- | --- |
| `init` | `{config, store}` |
| `shelf list` | `[{name, path, description, configured, missing, guidance_available, required, retention}]` |
| `shelf add` | `{name, path}` |
| `add` | `{id, path}` |
| `list`, `search` | `[{id, path, title, tags, metadata, errors}]` |
| `show` | `{id, path, content}` (complete Markdown, including frontmatter) |
| `context` | `{name, path, description, required, retention, guidance}` |
| `open` | `{paths: [...], opened: true}` after editor success |
| `validate` | `[{id, path, errors, valid}]` |
| `prune` | `[{id, path, status, error?}]` |

`metadata` is a JSON representation of YAML frontmatter, including unknown fields; YAML values that cannot be represented in JSON produce `null`. `errors` is an array of diagnostic strings. Display titles fall back to filename stems. Search/list warn about invalid files but succeed so other bits stay accessible.

`context.guidance` is `{path, text}` or `null`. A missing configured shelf produces a validation row with `id: null`. Prune statuses are `would_remove`, `removed`, or `skipped`; skipped rows include `error`. Future/permanent bits are not prune results. `--dry-run` changes only eligible rows' status, not selection rules.

Validation and prune may emit a complete result document and still exit 1. Early operational/usage errors emit only stderr (no partial JSON). Always check exit status. Editor stdout is redirected to stderr in JSON mode. Interactive flags are incompatible with JSON. Completion prints shell source, not JSON, and rejects `--json`.

Exit codes: **0** success (including no search matches), **2** CLI usage errors, **1** operational or validation failures.
