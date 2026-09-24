# bitshelf

## Idea

Build **bitshelf**, a local, Markdown-first snippet store with a small CLI named `bs` that agents and people can use independently of GitHub or any particular agent harness.

The store should support both:

- **Reusable snippets:** components, code patterns, explanations, and implementation notes.
- **Temporary snippets:** exact prompts, conversation handoffs, and scratch notes that expire automatically.

For example, preserve the multi-select date calendar built for dashboard filters in a `ui` collection, even if Invoice Date returns to the ordinary textbox multiselect. Save its implementation and integration details before removing it from the application.

Another example: save the original snippet-tool design prompt verbatim in a `tmp` collection. In another conversation, ask an agent to find the temporary snippet about the snippet tool and continue from there.

## Storage and configuration

Use ordinary Markdown files with YAML frontmatter. Files remain directly readable and editable, with no database required as the source of truth.

Follow XDG conventions for configuration. XDG defines filesystem locations, not a configuration language; use **TOML** for the config file.

Default config location:

```text
$XDG_CONFIG_HOME/bitshelf/config.toml
```

When `XDG_CONFIG_HOME` is unset, use `~/.config/bitshelf/config.toml`.

Example configuration:

```toml
store = "~/snippets"

[collections.ui]
required = ["title", "tags"]
description = "Reusable UI components and patterns"

[collections.tmp]
required = ["title"]
retention = "14d"
```

The destination is configurable. With this configuration, files would look like:

```text
~/snippets/
  ui/
    multiselect-date-calendar.md
  tmp/
    snippet-tool-design-prompt.md
```

Collections define organizational boundaries, required frontmatter properties, and optional retention policies. Validate basic property types as well as their presence. Keep the initial schema small rather than building a general-purpose schema language.

## Snippet format

Example snippet, with an illustrative generated timestamp:

````markdown
---
title: Multi-select date calendar
created: 2026-09-25T14:30:00Z
tags: [react, mui, calendar, multiselect]
---

## Purpose

Select independent dates across months, without restricting the selection to a range.

## Dependencies

MUI X Date Pickers and Dayjs.

## Implementation

```tsx
// Component implementation goes here.
```

## Integration notes

Describe controlled values, the popover anchor, applying changes on close,
focus restoration behavior, and Carbon-specific calendar spacing.
````

Generate creation timestamps and filename slugs automatically. Never silently overwrite an existing snippet.

For code snippets, preserve enough context to reuse the implementation: dependencies, relevant call-site code, assumptions, and noteworthy styling or behavior. An isolated component file may not be sufficient.

For exact-prompt snippets, preserve the supplied body verbatim rather than summarizing or rewriting it.

## CLI

The command is `bs`:

```sh
bs add ui --title "Multi-select date calendar" \
  --tags react,mui,calendar --file component.md

bs add tmp --title "Snippet tool design prompt" --stdin

bs search "snippet tool prompt" --collection tmp
bs show tmp/snippet-tool-design-prompt
bs list ui --tag calendar
bs validate
bs prune tmp --dry-run
```

### Initial commands

- **add:** Create a snippet, generate metadata, and enforce collection configuration.
- **list:** Browse snippets, optionally filtering by collection or tag.
- **search:** Search titles, tags, and bodies using plain-text search initially.
- **show:** Retrieve a snippet by its collection-qualified identifier.
- **validate:** Check directly edited files against configured requirements.
- **prune:** Remove expired snippets from collections with retention enabled; support dry-run previews.

Support stdin for content ingestion and `--json` output for reliable agent integration. Return clear errors when configuration or snippet metadata is invalid.

Collection-qualified identifiers such as `ui/multiselect-date-calendar` give agents a stable way to retrieve a search result without relying on its title being unique.

## Temporary collections and cleanup

A collection can opt into expiration using a retention setting such as `14d`.

When a temporary snippet is created, generate an explicit `expires` timestamp in its frontmatter. This makes its lifecycle visible in the file itself.

Run cleanup periodically through an operating-system scheduler rather than requiring a continuously running snippet service. Restrict automatic deletion to configured expiring collections, and provide `bs prune --dry-run` so users can inspect what would be removed.

Permanent collections must not be affected by temporary-snippet cleanup.

## Agent integration

Start with a thin agent skill documenting the CLI and expected workflows. A custom agent tool or server is not necessary for the first version.

An agent should be able to:

1. Discover configured collections and their requirements.
2. Save code, notes, or exact prompts with appropriate titles and tags.
3. Search for relevant snippets in a later conversation.
4. Retrieve the full content by identifier.

Keep the CLI usable outside an agent as well. Markdown files and structured CLI output avoid coupling the store to a particular agent product.

## Possible QMD integration

Consider integrating with [QMD](https://github.com/tobi/qmd) later for richer retrieval over the Markdown store.

Keep the Markdown files as the source of truth so an external search index can be added or rebuilt without changing the storage format. Plain-text search is sufficient for v1; QMD should not be a prerequisite.

## Recommended scope

Build the smallest useful version first:

- XDG-located TOML configuration.
- Configurable storage root and named collections.
- Markdown files with validated YAML frontmatter.
- Generated timestamps and safe filename slugs.
- `add`, `list`, `search`, `show`, `validate`, and `prune` commands.
- Stdin ingestion and JSON output for agents.
- Opt-in retention and scheduled cleanup for temporary collections.
- A thin skill explaining how agents should use the CLI.

Defer custom tools, servers, advanced schema machinery, and semantic search until the basic file-based workflow proves useful.
