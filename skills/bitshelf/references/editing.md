# Editing details

Use for metadata changes, body edge cases, and unusual IDs during `add`, `edit`, or `move`.

## Body input

- `add` needs a body source: `--file <PATH>`, `--file -`, or `--stdin`. Use `--file /dev/null` for an intentionally empty bit.
- The input is the body only; frontmatter in the file stays in the body rather than being merged into metadata.

## Metadata

- `--title` and `--tags` replace those fields.
- `--set <KEY=VALUE>` assigns a literal string, `--set-json <KEY=JSON>` a typed value, and `--unset <KEY>` removes a field. `edit` and `move` both accept these flags.
- Duplicate or conflicting operations on the same key fail.
- The CLI preserves unrelated metadata and `expires`; moves keep expiration too.

## Timestamps

`created` and `updated` are command-managed: leave them out of `--set`/`--set-json`. The one exception is `--unset` to remove an invalid `created`/`updated` value that blocks edits. A no-op edit keeps both dates. Direct filesystem edits leave dates to the user. There is no sync and no timestamp history.

## IDs

- Use IDs exactly as listed; a variant spelling (such as different letter case) is refused, not resolved.
- For a hyphen-prefixed positional ID, put options first and use `--`: `bs show --json -- -notes/bit`, `bs move --dry-run -- -notes/bit notes/bit`.
- For a hyphen-prefixed option value, use `=`: `--shelf=-notes`. Built-in aliases follow this convention; helper aliases forward arguments literally.

## Exit codes and validation

- An unknown built-in option exits 2.
- `bs validate --json` without a shelf checks every shelf, including non-discoverable ones; `validate` has no `--all`.
