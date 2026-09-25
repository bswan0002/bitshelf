# Shelves, bits, and configuration

## The store

A **store** is a directory of **shelves**. Each shelf is a directory containing its **bits** (Markdown files), its settings, and optionally guidance and other files:

```text
~/bitshelf/ui/
├── bs.toml             # shelf settings
├── SHELF.md            # optional guidance
├── bits/
│   └── command-menu.md
└── scripts/            # optional, ordinary shelf-local files
    └── import.py
```

Shelves are non-hidden top-level directories containing `bits/` or `bs.toml`, including ones you create in a file manager. There is no global registry: deleting a shelf's directory removes it from discovery. A shelf with `bs.toml` but no `bits/` is reported as missing its bits directory, and `bs shelf add NAME` repairs it.

Everything in a shelf other than bits, `bs.toml`, and `SHELF.md` is ignored by `bs` and preserved by content operations. The hidden store-level `.bitshelf/` directory holds [timestamp tracking state](timestamps.md), not a search index.

## Bits and identifiers

Bits are direct, non-hidden `.md` files inside `bits/`. A bit's ID is `shelf/name`, from `<store>/<shelf>/bits/<name>.md`, without the `.md` suffix or the `bits/` component. There are no nested bits and no separate UUIDs.

- Supply IDs directly to `bs add`. Names are never generated from titles.
- Renaming a file changes its ID immediately.
- Spaces and dots in names are supported; quote such IDs in the shell.

Each bit is YAML frontmatter followed by a Markdown body:

```markdown
---
title: Command menu
tags: [react, "project:bitshelf"]
created: 2026-09-25T22:41:18Z
updated: 2026-09-25T22:41:18Z
---
The body, stored exactly as supplied.
```

Built-in fields are `title` (optional string), `tags` (list of strings), `created`/`updated` ([reserved timestamps](timestamps.md)), and `expires` (set on [retention shelves](pruning.md)). Unknown fields are kept. Built-in field types are checked, along with shelf requirements.

## Global configuration

`bs init` writes `$XDG_CONFIG_HOME/bitshelf/config.toml`, defaulting to `~/.config/bitshelf/config.toml`:

```toml
store = "~/bitshelf"
editor = ["code"]

[aliases]
recent = ["list", "--sort", "updated", "--reverse"]
```

- `bs init --store ~/bitshelf --editor code` sets up non-interactively; with no arguments in a terminal, `bs init` prompts. It creates a default `notes` shelf and never overwrites an existing configuration.
- Use `--config PATH`, before or after a subcommand, to select a different configuration and store.
- A leading `~/` expands to `HOME`. Other relative paths, including `init --store` values, resolve against the configuration file's directory.
- Invalid or unknown settings fail clearly. To remove an editor setting, edit the TOML directly.
- See [editors](editing.md#editors) and [aliases](moving.md#aliases) for those settings.

## Shelf settings: `bs.toml`

```toml
description = "Reusable UI patterns"
required = ["title", "tags"]
# retention = "14d"      # optional; omit for permanent storage
# discoverable = false   # optional; defaults to true
```

| Setting | Meaning |
| --- | --- |
| `description` | Shown in `bs shelf list` and context. Helps people and agents pick the right shelf |
| `required` | Built-in fields that must be present: `title`, `tags`, `created`, `updated`, `expires`. Required `tags` means the list must exist; it may be empty |
| `retention` | Positive whole days (`14d`). New bits get `expires`. See [Expiration and pruning](pruning.md) |
| `discoverable` | `false` leaves the shelf out of default list/search and `open --pick`. See [the archive recipe](../recipes/archive.md) |
| `[tag_rules.*]` | Namespaced tag restrictions (below) |

`bs shelf add NAME [--description …] [--required …] [--retention …]` creates `bits/` and writes `bs.toml`, preserving existing contents and settings except those you pass. Edit the TOML directly to remove retention or manage tag rules. Saving shelf configuration normalizes its TOML formatting and comments. A shelf without `bs.toml` uses defaults. In `bs shelf list --json`, `configured` means `bs.toml` exists and `missing` means `bits/` is absent or not a directory.

## Namespaced tag rules

Tags are ordinary strings. A shelf can restrict tags in a *namespace* such as `project:`:

```toml
[tag_rules.project]
required = true
allowed = ["bitshelf", "switchboard", "data-explorer"]
```

```yaml
tags: ["project:bitshelf", "rust", "kind:guide"]
```

- `required = true` requires at least one tag in the namespace. It defaults to false.
- Every tag in a configured namespace must use an allowed value. Several allowed values on one bit are fine (`project:bitshelf`, `project:switchboard`).
- Ordinary tags and unconfigured namespaces are unrestricted.
- Matching is exact and case-sensitive, with no normalization. An empty value such as `project:` is invalid in a configured namespace.
- `allowed` is mandatory, nonempty, and free of duplicates. Namespace names and values must be nonempty and contain no whitespace, control characters, colons, or commas. Unknown rule settings are rejected.
- `required = ["tags"]` checks only that the field exists; namespace requirements are separate.

`bs add` and `bs edit` reject invalid tags. Files edited directly remain readable, and `bs validate` reports their violations. Changing rules never rewrites bits. `bs context SHELF --json` exposes the complete `tag_rules`, so authors can check requirements before writing. Filter with `bs list SHELF --tag project:bitshelf`.

## Shelf guidance: `SHELF.md`

`SHELF.md` sits at the shelf root, beside `bits/`, and tells whoever adds to the shelf, person or agent, how its contents should be written, maintained, and found. It can reference helper scripts kept in the shelf.

```sh
bs context ui
bs context ui --json
```

Context returns the guidance's **full text**, plus the description, requirements, tag rules, retention, the discovery setting, the shelf root `path`, and `bits_path`. Resolve shelf-relative paths in guidance against `path`. Missing guidance is an explicit `null`; unreadable guidance is an error.

Guidance is never treated as a bit: it's excluded from metadata validation, search, completion, and cleanup. Validation still checks that it isn't a symlink. `bs` doesn't enforce guidance or run the scripts it mentions. See [Guidance versus enforcement](../agents.md#guidance-versus-enforcement).

## Validation

```sh
bs validate            # all shelves
bs validate notes --json
```

Validation checks metadata types, shelf requirements, tag rules, shelf configuration, and managed-path symlinks, and reports every problem it finds. Malformed bits stay available through `show` and `open`, and listings include diagnostics rather than hiding other files.
