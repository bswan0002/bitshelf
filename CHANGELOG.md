# Changelog

## 0.1.0 — First public release candidate (not yet published)

- Reject attached values on value-less flags (including `--yes=false`) with exit 2 before mutations. Recognize BOM-prefixed frontmatter without nesting it on edits. Reject ambiguous binary, leading-zero and underscore-separated integers rather than silently coercing metadata.
- `bs add` requires `--file`, `--stdin` or `--interactive` instead of silently saving an empty body. IDs and shelf names must match stored names exactly; case/normalization variants are refused with the stored name. `bs edit ID --unset created|updated` removes an invalid reserved date without an editor.
- Prune considers only retention shelves: invalid bits on permanent shelves no longer make it fail, and each bit gets at most one row. Root-level links to files are ignored by discovery. Closed output pipes no longer mask failing exit statuses.
- `bs shelf add` leaves an unchanged `bs.toml` untouched and edits it in place, preserving comments, and refuses to recreate a missing store. Retention is limited to plain positive days up to `36500d`; blank names, non-regular bit files and empty or relative `XDG_CONFIG_HOME` are handled explicitly.

- Store notes, snippets, prompts and handoffs as ordinary UTF-8 Markdown in self-contained shelves, with optional titles, tags, shelf-local requirements and guidance.
- Search IDs, titles, tags and bodies with ranked AND/OR terms, phrases, exclusions and deterministic ordering. List IDs, exact paths or JSON; retrieve exact bodies with `show --body`.
- Save through stdin/files or waiting editor drafts. Markdown is authoritative: add owns creation dates, edits compare only current source and candidate, plain moves preserve dates, and external edits leave date management to the author.
- Add, edit and move share literal `--set`, typed `--set-json`, `--unset`, title and tag replacement. Destination tag rules are validated before a move without temporarily rewriting the source.
- Strict supported YAML values survive JSON and mutations without silent conversion loss. Unsupported keys, duplicates, tags, merges, non-finite numbers and overflowing integers fail clearly.
- Healthy shelves remain accessible during partial discovery. List/search/shelf-list JSON is `{results, errors, complete}`; incomplete scans exit 1. Helpers own their output contract and receive literal argv/configuration context.
- Guarded no-clobber creation, atomic replacement, OS writer locks with bounded waiting, recoverable drafts, destination-first moves, and conservative explicit-expiration pruning with per-item partial outcomes.
- Shell completion setup updates resolved startup files atomically. The package contains the executable, man page, bash/zsh/fish completions, matching agent skill and third-party notices.
- Unknown built-in options fail with exit 2 before writes. Hyphen-prefixed positional operands require `--`; explicit option values such as `--shelf=-notes` remain supported. Built-in ID aliases preserve operand/value roles during substitution; external helper aliases retain literal argv forwarding.
- Breaking from the unpublished prototype: remove `sync` and persistent timestamp history; ignore old state files without migration or deletion; change collection JSON to completeness envelopes; reject unsupported discovered names. Back up store/configuration before trying a different minor release.
- Scope and limits: local trusted macOS/Linux filesystems; best-effort external-editor conflict checks, no distributed lock or multi-file transaction, no attachment/link rewriting. Frontmatter presentation can normalize; exact UTF-8 bodies and supported values are preserved. macOS downloads are unsigned and unnotarized; signing and independent attestations are deferred. Checksums are integrity checks, not independent provenance.
