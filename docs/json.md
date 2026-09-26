# JSON contract (0.1.x)

`--json` is global. For list/search, it conflicts with `--long`, `--paths`, and `--null`. Successful structured output is one document on stdout; diagnostics go to stderr. List/search/shelf-list use `{results, errors, complete}`; results is an array, including `[]`. Validation/prune use per-item arrays. Paths are absolute. Nullable fields are emitted as `null`, not omitted. Consumers should tolerate new fields.

| Command | Result |
| --- | --- |
| `init` | `{config, store}` |
| `shelf list` | `{results: [{name, path, description, configured, missing, discoverable, guidance_available, required, retention}], errors, complete}` |
| `shelf add` | `{name, path}` |
| `add` | `{id, path}` |
| `move` | `{from, id, path, dry_run}` (destination ID/path; also used by move aliases) |
| `aliases` | Object mapping names to built-in argument arrays or helper objects `{exec: [...]}` |
| `edit` | `{id, path, changed}` |
| `list` | `{results: [{id, path, title, tags, metadata, errors}], errors, complete}` |
| `search` | `{results: [{id, path, title, tags, metadata, errors, matches: {fields, score}}], errors, complete}` |
| `show` | `{id, path, content}` (complete Markdown by default; body only with `--body`) |
| `context` | `{name, path, bits_path, description, discoverable, required, tag_rules, retention, guidance}` |
| `open` | `{paths: [...], opened: true}` after editor success |
| `validate` | `[{id, path, errors, valid}]` |
| `prune` | `[{id, path, status, error?}]` |

`metadata` is a JSON representation of YAML frontmatter, including unknown fields; unsupported YAML values produce explicit errors (see [frontmatter](concepts/frontmatter.md)). `errors` is an array of diagnostic strings. `title` is the nonempty title metadata string, or `null` when absent/invalid; it is not synthesized from the filename. Use `id` for display and identity. Search/list include accessible invalid bits with per-bit errors, preserve healthy results, set complete=false, emit scope errors `{path, error}`, and exit 1. Unreadable/unsupported paths are scope errors, never fabricated empty bits.

**Integer precision:** bs preserves all i64/u64 integers exactly. JavaScript's
`JSON.parse` cannot represent every integer beyond ±(2^53−1); converting to
`BigInt` afterward cannot recover lost digits. Use a lossless JSON parser when
exact large values matter, or store large identifiers as strings.

Search's `matches.fields` lists positive-term matches in stable order: `id`, `title`, `tags`, `body` (only matching fields are included). `matches.score` is an implementation-defined ranking signal. Results are score-descending with ID tie-breaking unless `--sort id` is selected. Exclusion-only matches have `fields: []` and `score: 0`. Bodies and snippets are not emitted; retrieve selected bits with `show`. See [search syntax](concepts/finding.md#search).

`context.guidance` is `{path, text}` or `null`. `context.path` is the shelf root; `bits_path` is its content directory. Bit IDs remain `shelf/bit-name`, while bit paths include `bits/`. In shelf listings, `configured` means local `bs.toml` exists and `missing` means the bits directory is absent or not a directory. Structural validation errors (missing bits directories, invalid shelf configuration, or symlinked shelves/settings/guidance) produce rows with `id: null`. Symlinked bits produce rows with their bit IDs. Validation does not follow links and continues checking other accessible shelves/bits. Deleted shelves are no longer discovered. Prune statuses are `would_remove`, `removed`, `removed_with_error`, or `skipped`; skipped rows include `error`. Future bits and bits on permanent shelves (valid or invalid) are not prune results; each retention-shelf bit produces at most one row, and `id: null` rows are scopes or entries whose retention or ID is unknown. `--dry-run` changes only eligible rows' status, not selection rules.

Validation and prune may emit a complete result document and still exit 1. Operational/usage errors outside per-item results emit only stderr (no partial JSON). An error can occur after a file was saved; diagnostics distinguish saved bits with failed durability from unsuccessful saves. Inspect the destination and follow the recovery instructions instead of blindly retrying creation. Always check exit status. Editor stdout is redirected to stderr in JSON mode. Interactive flags are incompatible with JSON. Completion prints shell source, not JSON, and rejects `--json`.

Unknown options fail with exit 2 before content operations; hyphen-prefixed positional operands require `--` (see [identifiers](concepts/identifiers.md)).

Exit codes: **0** success (including no search matches and quietly closed output pipes), **2** CLI usage errors, **1** operational or validation failures.

`edit.changed` compares the current source and candidate body/user metadata, excluding created/updated and YAML formatting. Read-only commands do not change dates. Missing imported dates stay missing on no-ops and plain moves. See [timestamps](concepts/timestamps.md). `list --sort created|updated --reverse` sorts parsed dates, missing/invalid last, with deterministic ID tie-breaking.

`discoverable` defaults to true. Default list/search omit shelves where it is false; `--all` or an explicit shelf includes them. Maintenance commands still include these shelves. `validate` without a shelf checks all shelves, including non-discoverable ones; it has no `--all` option. Validation/prune keep per-item arrays because each row reports an item or scope outcome; check exit status as well as rows. Move dry runs return the planned destination without writing. Built-in aliases return their target command’s JSON result and exit status. Explicit helper aliases forward `--json` and preserve exit status, but the helper owns its output contract; bs does not enforce JSON output for arbitrary executables.

See [compatibility](compatibility.md) for incomplete discovery and release boundaries.
