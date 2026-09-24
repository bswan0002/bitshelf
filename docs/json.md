# JSON contract (0.1.x)

`--json` is global. Successful structured output is one document on stdout; diagnostics go to stderr. Arrays are used for collection results, including `[]`. Paths are absolute. Nullable fields are emitted as `null`, not omitted. Consumers should tolerate new fields.

| Command | Result |
| --- | --- |
| `init` | `{config, store}` |
| `shelf list` | `[{name, path, description, configured, missing, guidance_available, required, retention}]` |
| `shelf add` | `{name, path}` |
| `add` | `{id, path}` |
| `edit` | `{id, path, changed}` |
| `sync` | `{dry_run, results: [{id, changed, metadata_changed, baselined, error}]}` |
| `list`, `search` | `[{id, path, title, tags, metadata, errors}]` |
| `show` | `{id, path, content}` (complete Markdown, including frontmatter) |
| `context` | `{name, path, bits_path, description, required, retention, guidance}` |
| `open` | `{paths: [...], opened: true}` after editor success |
| `validate` | `[{id, path, errors, valid}]` |
| `prune` | `[{id, path, status, error?}]` |

`metadata` is a JSON representation of YAML frontmatter, including unknown fields; YAML values that cannot be represented in JSON produce `null`. `errors` is an array of diagnostic strings. Display titles fall back to filename stems. Search/list warn about invalid files but succeed so other bits stay accessible.

`context.guidance` is `{path, text}` or `null`. `context.path` is the shelf root; `bits_path` is its content directory. Bit IDs remain `shelf/slug`, while bit paths include `bits/`. In shelf listings, `configured` means local `bs.toml` exists and `missing` means the bits directory is absent or not a directory. Structural validation errors (missing bits directories, invalid shelf configuration, or symlinked shelves/settings/guidance) produce rows with `id: null`. Symlinked bits produce rows with their bit IDs. Validation does not follow links and continues checking other accessible shelves/bits. Deleted shelves are no longer discovered. Prune statuses are `would_remove`, `removed`, or `skipped`; skipped rows include `error`. Future/permanent bits are not prune results. `--dry-run` changes only eligible rows' status, not selection rules.

Validation, sync and prune may emit a complete result document and still exit 1. Operational/usage errors outside per-item results emit only stderr (no partial JSON). An error can occur after a file was saved; diagnostics distinguish saved bits with failed bookkeeping from unsuccessful saves. Inspect the destination and follow the recovery instructions instead of blindly retrying creation. Always check exit status. Editor stdout is redirected to stderr in JSON mode. Interactive flags are incompatible with JSON. Completion prints shell source, not JSON, and rejects `--json`.

Exit codes: **0** success (including no search matches), **2** CLI usage errors, **1** operational or validation failures.

`created` and `updated` in bit metadata are reserved CLI-managed RFC 3339 timestamps. Generated dates use UTC; equivalent valid imported representations are preserved. `edit.changed` indicates a body/user-metadata change relative to the tracking baseline, excluding reserved fields and YAML formatting. Sync uses the same `changed` meaning; `metadata_changed` indicates that timestamp reconciliation changes the Markdown file, and `baselined` indicates first-time tracking. On dry runs these describe planned changes only; nothing is written. `error` is null on success or an error string on failure. Existing valid dates survive baselining; missing dates use first-tracking time. Read-only commands never reconcile dates. `list --sort created|updated --reverse` orders by parsed timestamp, missing/invalid dates last, with deterministic identifier tie-breaking.
