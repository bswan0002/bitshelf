# Changelog

## 0.1.0 — Unreleased prototype

- Rust `bs` CLI with Usage declarations, generated help/reference/man page, and self-contained dynamic completion.
- TOML/XDG configuration, discoverable shelves, verbatim Markdown ingestion, frontmatter validation, plain-text search, and JSON results.
- Configured/environment editors, Demand setup, draft authoring, and filtered multi-open selection.
- Init honors VISUAL/EDITOR without prompting; recognized GUI editors automatically wait for draft editing only.
- Automatic reserved `created`/`updated` timestamps, editor or body-input `bs edit`, hash-based `bs sync` for external edits, and chronological list sorting.
- Patched Demand's duplicate completed-prompt rendering, covered by real-terminal regression tests.
- Self-contained shelves with `bits/` content, local `bs.toml` settings, optional `SHELF.md`, and readable `shelf/bit-name` identifiers.
- ID-first creation (`bs add shelf/bit-name`) with optional title metadata; no generated slugs.
- ID-only list/search defaults, `--long`, `--paths`, and `--null` output, and ID-aware search.
- Body-only retrieval (`show --body`), conventional stdin input (`add/edit --file -`), and quiet broken-pipe handling.
- Shelf guidance context and portable bitshelf agent skill, including a recipe for ordinary shelf-local helpers referenced from guidance.
- Explicit opt-in expiration with safe dry-run cleanup.
- Temporary-store contract tests, VitePress docs, CI, and release/Homebrew automation scaffolding.
