# JSON contract (0.1.x)

`--json` is global. For list/search, it conflicts with `--long`, `--paths`, and `--null`. Successful structured output is one document on stdout; diagnostics go to stderr. Arrays are used for collection results, including `[]`. Paths are absolute. Nullable fields are emitted as `null`, not omitted. Consumers should tolerate new fields.

| Command | Result |
| --- | --- |
| `init` | `{config, store}` |
| `shelf list` | `[{name, path, description, configured, missing, discoverable, guidance_available, required, retention}]` |
| `shelf add` | `{name, path}` |
| `add` | `{id, path}` |
| `move` | `{from, id, path, dry_run}` (destination ID/path; also used by move aliases) |
| `aliases` | Object mapping alias names to argument arrays |
| `edit` | `{id, path, changed}` |
| `sync` | `{dry_run, results: [{id, changed, metadata_changed, baselined, error}]}` |
| `list` | `[{id, path, title, tags, metadata, errors}]` |
| `search` | `[{id, path, title, tags, metadata, errors, matches: {fields, score}}]` |
| `show` | `{id, path, content}` (complete Markdown by default; body only with `--body`) |
| `context` | `{name, path, bits_path, description, discoverable, required, tag_rules, retention, guidance}` |
| `open` | `{paths: [...], opened: true}` after editor success |
| `validate` | `[{id, path, errors, valid}]` |
| `prune` | `[{id, path, status, error?}]` |

`metadata` is a JSON representation of YAML frontmatter, including unknown fields; YAML values that cannot be represented in JSON produce `null`. `errors` is an array of diagnostic strings. `title` is the nonempty title metadata string, or `null` when absent/invalid; it is not synthesized from the filename. Use `id` for display and identity. Search/list warn about invalid files but succeed so other bits stay accessible.

Search's `matches.fields` lists positive-term matches in stable order: `id`, `title`, `tags`, `body` (only matching fields are included). `matches.score` is the sum of each distinct positive term's strongest field weight: ID/title 8, tags 4, body 1. Results are score-descending with ID tie-breaking unless `--sort id` is selected. Exclusion-only matches have `fields: []` and `score: 0`. Bodies and snippets are not emitted; retrieve selected bits with `show`. See [search syntax](concepts/finding.md#search).

`context.guidance` is `{path, text}` or `null`. `context.path` is the shelf root; `bits_path` is its content directory. Bit IDs remain `shelf/bit-name`, while bit paths include `bits/`. In shelf listings, `configured` means local `bs.toml` exists and `missing` means the bits directory is absent or not a directory. Structural validation errors (missing bits directories, invalid shelf configuration, or symlinked shelves/settings/guidance) produce rows with `id: null`. Symlinked bits produce rows with their bit IDs. Validation does not follow links and continues checking other accessible shelves/bits. Deleted shelves are no longer discovered. Prune statuses are `would_remove`, `removed`, or `skipped`; skipped rows include `error`. Future/permanent bits are not prune results. `--dry-run` changes only eligible rows' status, not selection rules.

Validation, sync and prune may emit a complete result document and still exit 1. Operational/usage errors outside per-item results emit only stderr (no partial JSON). An error can occur after a file was saved; diagnostics distinguish saved bits with failed bookkeeping from unsuccessful saves. Inspect the destination and follow the recovery instructions instead of blindly retrying creation. Always check exit status. Editor stdout is redirected to stderr in JSON mode. Interactive flags are incompatible with JSON. Completion prints shell source, not JSON, and rejects `--json`.

Exit codes: **0** success (including no search matches and quietly closed output pipes), **2** CLI usage errors, **1** operational or validation failures.

`created` and `updated` in bit metadata are reserved CLI-managed RFC 3339 timestamps. Generated dates use UTC; equivalent valid imported representations are preserved. `edit.changed` indicates a body/user-metadata change relative to the tracking baseline, excluding reserved fields and YAML formatting. Sync uses the same `changed` meaning; `metadata_changed` indicates that timestamp reconciliation changes the Markdown file, and `baselined` indicates first-time tracking. On dry runs these describe planned changes only; nothing is written. `error` is null on success or an error string on failure. Existing valid dates survive baselining; missing dates use first-tracking time. Read-only commands never reconcile dates. `list --sort created|updated --reverse` orders by parsed timestamp, missing/invalid dates last, with deterministic identifier tie-breaking.

`discoverable` defaults to true. Default list/search omit shelves where it is false; `--all` or an explicit shelf includes them. Maintenance commands still include these shelves. Move dry runs return the planned destination without writing. Configured aliases return their target command’s JSON result and exit status.
