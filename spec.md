# bitshelf specification

Status: initial implementation specification, consolidated from `idea.md` and the design discussion. This document describes the intended application; the commands below are not yet implemented. Build the application in a follow-up session.

## Purpose

**bitshelf** is a local, Markdown-first store for notes, snippets, prompts, and other reusable or temporary material. Its executable is **`bs`**. People and agents are equal users of the same files and commands.

The central promise: **your store remains a useful directory of Markdown files even without bitshelf installed.** You can browse it, edit it in your preferred editor, search it with ordinary tools, back it up, or put it in Git.

Storage is explicit: users decide what to save, where it belongs, what metadata matters, and whether it expires. No background agent-memory capture is required. The application is independent of GitHub, any particular editor, and any agent harness.

## Terminology and examples

- **Store:** the configurable root directory containing all shelves.
- **Shelf:** a top-level directory grouping related material. This is the collection described in `idea.md`; use `shelf` consistently in the CLI and configuration.
- **Bit:** an individual Markdown file within a shelf.
- **Identifier:** a store-relative, shelf-qualified name without `.md`, such as `ui/multiselect-date-calendar`.

Typical uses:

- Preserve a multi-select date calendar before removing it from an application. Save dependencies, implementation, call-site examples, and integration details in the `ui` shelf.
- Keep reusable notes, checklists, explanations, code patterns, and prompts in permanent shelves.
- Save an original design prompt verbatim in `tmp`, then retrieve it from another conversation.
- Store conversation handoffs and scratch notes with explicit expiration.
- Open one bit, several bits, a shelf, or the entire store in a preferred text editor.

## Storage layout

The proposed default store is `~/bitshelf`; users can select another location.

```text
~/bitshelf/
  notes/
    bs.toml
    bits/
      release-checklist.md
  ui/
    bs.toml
    SHELF.md
    bits/
      command-menu.md
    scripts/
      import.py
```

Markdown files are the source of truth. No database, background service, or search index is required. Any future index must be disposable and rebuildable from the files.

Bits live directly inside each shelf's `bits/` directory. Non-hidden top-level directories containing `bits/` or `bs.toml` are discoverable shelves. Missing `bs.toml` means default settings. Shelves with settings but without a bits directory are reported as missing their bits directory, repairable with `bs shelf add`. There is no global shelf registry; deleted shelves disappear from discovery.

Root-level `SHELF.md` is optional guidance, not a bit. `bs context` exposes it explicitly. Other shelf-local files and directories (including `scripts/`) are ignored and preserved by bit operations. A Markdown file named `SHELF.md` inside `bits/` is an ordinary bit.

Directly creating, editing, renaming, or deleting files is a supported workflow. Identifiers remain `shelf/slug`, omitting the `bits/` path component and `.md` suffix: renaming a file changes its identifier. No separate registry needs updating.

## Configuration

Use TOML with XDG configuration locations:

```text
$XDG_CONFIG_HOME/bitshelf/config.toml
```

If `XDG_CONFIG_HOME` is unset, use `~/.config/bitshelf/config.toml`. XDG determines the location; TOML determines the file syntax. Use these defaults consistently, with `--config PATH` available for an explicit override.

```toml
store = "~/bitshelf"

# An executable followed by arguments; append selected file/directory paths.
editor = ["code", "--reuse-window"]

```

Each shelf has its own optional `bs.toml`, containing settings directly (no `[shelves.*]` wrapper). For example, `tmp/bs.toml`:

```toml
description = "Temporary prompts, handoffs, and scratch notes"
required = ["title"]
retention = "14d"
```

`bs init` creates `notes/bits/` and `notes/bs.toml`. `bs shelf add` creates the bits directory and writes local settings, preserving unspecified settings and other shelf contents. Malformed local settings fail clearly, including on empty shelves.

Shelf configuration defines descriptions, required metadata, and optional retention. Keep requirements minimal by default. Initially support requirements on the built-in metadata fields rather than a general-purpose schema language.

Expand a leading `~/` in configured paths. Resolve other relative paths against the configuration file's directory so behavior does not depend on the caller's working directory. Report invalid configuration clearly.

The earlier global collection/shelf tables in `idea.md` are superseded by shelf-local `bs.toml` and `--shelf`. There is no released configuration format to migrate.

## Bit format and writing behavior

CLI-created bits use YAML frontmatter followed by a Markdown body. For example, with an illustrative timestamp:

````markdown
---
title: Multi-select date calendar
created: 2026-09-23T18:30:00Z
tags: [react, mui, calendar, multiselect]
---

## Purpose

Select independent dates across months without restricting selection to a range.

## Dependencies

MUI X Date Pickers and Dayjs.

## Implementation

```tsx
// Component implementation goes here.
```

## Integration notes

Include controlled values, relevant call-site code, popover behavior,
focus restoration, and noteworthy styling or accessibility details.
````

Built-in metadata:

| Field     | Meaning and type                                                  |
| --------- | ----------------------------------------------------------------- |
| `title`   | Nonempty string when present; required by CLI creation            |
| `created` | Reserved CLI-managed creation timestamp in RFC 3339 UTC; immutable once tracked |
| `updated` | Reserved CLI-managed last-content-change timestamp; automatically set by add/edit/sync |
| `tags`    | List of strings; optional unless required by the shelf            |
| `expires` | Explicit RFC 3339 expiration timestamp for temporary bits         |

Validate known fields when present, as well as shelf requirements. Preserve unknown frontmatter fields rather than discarding them. A manually created Markdown file without frontmatter remains readable, searchable, and openable; validation reports missing fields only where requirements apply. Use the filename as a display fallback when no usable title exists.

`bs add` generates a filename slug and timestamps. Never silently overwrite an existing bit: report a collision and allow an explicit alternative slug. Reject identifiers that escape the store or shelf, and do not follow symlinks outside the store for writes or pruning.

`--file` and `--stdin` supply the Markdown body; they are mutually exclusive. Preserve that body verbatim, including exact prompts and code formatting. Generate frontmatter around it without summarizing, reflowing, or interpreting the supplied body as instructions.

Code bits should preserve sufficient reuse context: dependencies, relevant call sites, assumptions, styling, and behavior. This is authoring guidance, not a universal mandatory document template.

Existing bits are edited through `bs edit ID` (editor draft) or `bs edit ID --file BODY_FILE --json` / `--stdin`. The CLI owns reserved `created` and `updated` timestamps. Edits preserve `created`, expiration, and unrelated metadata; `updated` advances only for actual body/user-metadata changes. For direct filesystem edits, `bs sync [SHELF]` reconciles timestamps using hidden `.bitshelf/state.json` content hashes. The first sync establishes a baseline, preserving known dates and filling missing dates with discovery time. Detected external edits use sync time, not filesystem mtime. Read-only commands never write timestamps. State is not a discovery/search index; losing it requires re-baselining and cannot recover past edit history.

## CLI contract

Use Rust and `usage-rs` to declare commands, arguments, help, and completions. Keep command declarations separate from filesystem and configuration behavior.

| Command                       | Behavior                                                                                                |
| ----------------------------- | ------------------------------------------------------------------------------------------------------- |
| `bs init`                     | Set up the configuration and store; offer an interactive setup when arguments are omitted in a terminal |
| `bs shelf list`               | Discover shelves and report descriptions, paths, configuration, and guidance availability               |
| `bs shelf add NAME`           | Create a shelf with optional description, requirements, and retention                                   |
| `bs add SHELF`                | Create a bit with generated metadata and validate it against shelf requirements                         |
| `bs list [SHELF]`             | List bits, optionally filtered by tag                                                                   |
| `bs search QUERY`             | Search titles, tags, and bodies using plain-text matching                                               |
| `bs show ID`                  | Print a bit's complete Markdown content                                                                 |
| `bs edit ID` | Edit a bit, automatically managing timestamps; body replacement via `--file` / `--stdin` |
| `bs sync [SHELF]` | Reconcile timestamps after external edits; supports `--dry-run` |
| `bs open [TARGET...]`         | Open the store, a shelf, or one or more bits in the preferred editor                                    |
| `bs context SHELF`            | Return the shelf's authoring context, including full guidance text                                      |
| `bs validate [SHELF]`         | Report metadata and configuration violations without rewriting files                                    |
| `bs prune [SHELF]`            | Remove explicitly expired bits from shelves with retention enabled                                      |
| `bs completion --shell SHELL` | Print the Usage-generated completion script for a shell                                                 |

Representative commands:

```sh
bs add ui --title "Multi-select date calendar" \
  --tags react,mui,calendar --file component.md

bs add tmp --title "Snippet tool design prompt" --stdin

bs shelf list --json
bs context ui --json
bs search "snippet tool prompt" --shelf tmp --json
bs show tmp/snippet-tool-design-prompt
bs list ui --tag calendar
bs validate ui
bs prune tmp --dry-run
```

Provide an explicit noninteractive argument path for every interactive operation. Ordinary commands with complete arguments must not prompt. Missing required arguments in a noninteractive session must produce actionable errors instead of waiting for input.

Support `--json` for discovery, reading, context, validation, and mutation results. JSON output must be a single valid document with stable documented fields, never mixed with prompts, progress messages, or terminal styling. Collection results should consistently use arrays, including empty results. Include identifiers and absolute paths where relevant so agents can retrieve or edit a result without guessing.

Keep diagnostics on stderr. Use consistent exit statuses: `0` for success, `2` for command-line usage errors, and `1` for operational or validation failures. A search with no results is successful. Clearly distinguish malformed configuration, missing targets, invalid metadata, and filesystem errors.

Search and listing should use deterministic ordering. Invalid metadata in one bit should produce a diagnostic without making all other bits inaccessible. Raw reading and opening should continue to work for malformed files.

## Opening files and editor behavior

```sh
bs open
# Open the store directory.

bs open ui
# Open the ui shelf directory.

bs open ui/multiselect-date-calendar notes/release-checklist
# Open multiple individual files.

bs open --pick
# Interactively select bits to open.
```

Resolve the editor in this order: configured `editor`, then `$VISUAL`, then `$EDITOR`. If none is available, return an actionable setup error. Launch the executable with arguments and paths, without evaluating a shell command. Support conventional quoted arguments in editor environment variables without shell expansion or command substitution.

Pass directories to the editor for store/shelf opening and paths for individual bits. Directory support varies by editor; document that the configured editor must support the requested operation. Do not turn opening a directory into opening every file implicitly.

`open` launches an editor; `show` returns content. Agents retrieving text should use `show` rather than launching an editor. Let users configure editor flags such as `--wait` when appropriate.

## Human interaction with Demand

Use Demand for focused interactive terminal workflows:

- `bs init`: select the store location and preferred editor.
- `bs shelf add` without a name in a terminal: collect shelf settings.
- `bs add --interactive`: select a shelf, enter metadata, and open a draft in the editor.
- `bs open --pick`: filter and select multiple bits to open.

Demand supplies input validation, filtered selection, multiselect, and confirmations where an operation warrants one. Long-form content editing remains in the user's editor. Interactive add should validate the completed draft before finalizing it and preserve the draft if validation fails or the editor fails.

Require a terminal for interactive modes, and disable prompts in JSON and noninteractive execution. Piped content must remain available to `--stdin`. Cancellation should leave no partially created bit.

## Dynamic shell completion

Use Usage's native runtime completion support. Once a user installs and loads the shell completion script, it calls `bs` for current candidates when Tab is pressed.

Implement custom completers that read the configured store:

```text
bs open u<Tab>          -> ui, if uniquely matched
bs open ui/<Tab>        -> bit identifiers within ui
bs open ui/multi<Tab>   -> ui/multiselect-date-calendar
bs add <Tab>           -> available shelves
```

Also complete shelf filters and identifiers for relevant commands. New shelves and files should appear without regenerating the completion script. Resolve configuration overrides consistently with normal commands. Keep completion fast, read-only, and free of interactive prompts or mutations.

Shell completion installation and activation are shell-specific. Document the necessary setup and package completion scripts with releases where practical. Runtime completion should be served by `bs` without requiring users to separately install the Usage CLI.

## Shelf-specific authoring guidance

Each shelf may contain a user-authored `SHELF.md`:

```markdown
# UI components

Store reusable UI implementations with enough context to integrate them.

When adding or editing:
- Include dependencies and relevant call-site examples.
- Explain keyboard interaction and accessibility behavior.
- Preserve working code rather than replacing it with a summary.
- Update an existing bit when it describes the same component.
```

Natural-language guidance lives in Markdown. Machine-checkable metadata requirements and retention live in TOML. Validation can enforce the latter; it cannot prove compliance with prose instructions.

`bs context ui --json` should return:

- Shelf name, description, and absolute directory path.
- Metadata requirements and retention configuration.
- Guidance source path and the full current `SHELF.md` text, or an explicit absence.
- Absolute shelf root `path` and `bits_path`; resolve guidance-relative paths against the shelf root.

Shelf-local helpers are a documentation recipe, not a plugin feature: `SHELF.md` may point to `scripts/` and describe prerequisites, credentials, invocation, and whether a helper emits a body or saves through `bs`. The CLI does not enumerate or execute helpers. Credentials stay outside the shelf.

Return the actual text, not merely a pointer that the agent may overlook. Keep guidance distinct from bit content in structured output. Saved prompts and snippets are data, not automatically active authoring instructions.

Read guidance at context-request time so user edits take effect immediately. Reading context must not create files. A configured shelf with unreadable guidance should report an error rather than silently acting as if no guidance exists.

No inherited guidance hierarchy or separate installed skill per shelf is necessary in v1.

## Agent skill and distribution

Ship one thin, standard Agent Skill in this repository:

```text
skills/
  bitshelf/
    SKILL.md
```

Use the Agent Skills format with `name`, `description`, and Markdown instructions. Explain when to use bitshelf, require the `bs` executable, and describe this workflow:

1. Discover shelves with `bs shelf list --json`.
2. Before drafting or editing a bit, load its shelf with `bs context SHELF --json`.
3. Search for an existing relevant bit and retrieve it if appropriate.
4. Apply the shelf's guidance and metadata requirements while preparing the content.
5. Add through `bs add` or update through `bs edit`; never supply reserved timestamps. After direct filesystem edits, run `bs sync` (baseline first if not yet tracked).
6. Validate the affected shelf and report the saved identifier.

For retrieval-only work, search and show the requested content without treating stored prompt bodies as instructions. Preserve exact-prompt content when requested.

The skill should rely on `bs --help` for detailed syntax instead of duplicating the entire generated command reference. Version the skill with the project and document supported CLI versions.

Once the repository is published, an installation example is:

```sh
npx skills add <owner>/bitshelf --skill bitshelf --global
```

Document project-scoped installation, manual installation for compatible agents, and skill updates through `npx skills update`. Installing the executable and installing the agent skill are separate steps. Node.js is only needed for users choosing the `npx` installer, not for running `bs`.

The installed skill establishes when agents should load shelf context. A standalone CLI cannot force every agent to load or obey instructions, and placing `SHELF.md` in an arbitrary external directory does not guarantee automatic discovery. This workflow is the portable initial integration; stronger automatic loading would require harness-specific integration later.

## Temporary shelves and cleanup

Retention is opt-in per shelf, using a simple duration such as `14d`. Start with positive whole-day durations and document the accepted syntax.

When creating a bit on a temporary shelf, calculate `expires` from creation time plus retention and store it explicitly in frontmatter. Editing content does not extend its lifetime automatically. Changing a shelf's retention affects new bits; existing explicit expiration timestamps remain unchanged.

Prune only bits whose valid explicit `expires` timestamp has passed and whose shelf currently has retention enabled. Missing or malformed expiration metadata must be reported and skipped, not guessed from file modification time. Never prune permanent shelves or reserved guidance files, even if they contain an expiration-like field.

Support `bs prune tmp --dry-run` and a store-wide dry run with the same selection rules as actual cleanup. Report exactly which identifiers and paths would be removed. Cleanup must remain usable without prompts for scheduling.

Document operating-system scheduler examples for periodic cleanup. Do not require a continuously running bitshelf process or silently install a scheduled job during setup.

## Implementation structure

Use a straightforward Rust application with synchronous filesystem operations. Keep one package initially; a large workspace and asynchronous runtime are unnecessary for the proposed scope.

Suggested organization:

```text
Cargo.toml
src/
  main.rs
  cli/          # Usage declarations and command dispatch
  config.rs     # TOML, XDG locations, editor settings
  store.rs      # Shelf discovery, identifiers, file operations
  bit.rs        # Frontmatter, slugs, metadata validation
  context.rs    # Shelf guidance and authoring context
  editor.rs     # Editor process invocation
  interactive.rs
  output.rs     # Human and structured output
tests/
skills/bitshelf/SKILL.md
docs/
.github/workflows/
```

Use shared storage and validation functions from both interactive and noninteractive commands. Keep filesystem behavior independent of Usage and Demand so it can be tested without terminal interaction.

Test meaningful contracts: verbatim bodies, collision handling, malformed metadata, identifier containment, multiple editor arguments, dynamic completion, JSON output, guidance retrieval, and pruning boundaries. Use temporary stores and controlled clocks for tests; never test destructive behavior against a user's real shelf.

## Documentation, releases, and installation

Usage defines the CLI and generates supporting artifacts. Release automation, package management, and website hosting are separate project responsibilities.

Generate the command reference and man page from the same declarations used by the executable:

```sh
bs __usage_spec__ > bs.usage.kdl
usage generate markdown --file bs.usage.kdl --multi --out-dir docs/reference
usage generate manpage --file bs.usage.kdl --out-file bs.1
```

The Usage CLI is a development/documentation-generation dependency. Normal users should only need the installed bitshelf executable and their editor.

Use VitePress for a documentation website containing a short written guide alongside the generated reference. Explain setup, shelves, direct file editing, editor configuration, completions, temporary cleanup, and agent workflows. Publish through a separate GitHub Pages workflow. Clearly identify development docs if publishing from `main` ahead of a release.

Use GitHub Actions for tests and release builds. The intended release process is:

1. Update the version and changelog; publish a `v*` version tag.
2. Create a draft GitHub release.
3. Build binaries for the initial supported platform matrix.
4. Package the executable, man page, and completion resources; publish checksums.
5. Publish the release after required builds and checks succeed.
6. Update the project's Homebrew tap with the released version and checksums.

Start with macOS on Apple Silicon and Intel and Linux x86-64; expand the platform matrix when supported and tested. The build may use separate macOS archives or a universal binary. Windows distribution can follow without making it a prerequisite for the first useful release.

Maintain a personal Homebrew tap rather than depending on admission to Homebrew's official catalog. Proposed installation:

```sh
brew install <owner>/tap/bitshelf
bs --version
brew upgrade bitshelf
```

The formula is named `bitshelf`; the executable it installs is `bs`. Package shell completions and the man page where Homebrew expects them. Updates follow the installation method; do not add `bs self-update` initially. Provide downloadable binaries, with Cargo installation optional once published to crates.io and naming availability is confirmed.

Use the Usage repository's release pipeline as a reference, not a requirement to reproduce every job. Its example includes macOS signing/notarization, Packslip attestations, and enhanced release notes. Defer Packslip and automated release-note enhancement. Decide macOS signing/notarization before advertising browser-downloaded archives as a frictionless install route. Do not assume the commented-out Homebrew bump job in that example performs any updates.

## Delivery sequence and acceptance criteria

First milestone: **configure a store, save a bit, find it, and open it in an editor.** Implement the vertical workflow before adding release machinery.

Then complete the initial scope:

- Shelf configuration and discovery, minimal validated metadata, stdin ingestion, and JSON results.
- `open` for directories and multiple bits, plus Demand-based setup and selection.
- Runtime completion from actual shelf contents.
- `SHELF.md`, `bs context`, and the distributable bitshelf agent skill.
- Opt-in expiration, safe dry-run pruning, and scheduler documentation.
- Generated reference documentation, release binaries, and a Homebrew tap.

The initial version is ready when:

- A person can create and edit bits using only a text editor, then find and open them with `bs`.
- An agent can discover a shelf, load its guidance, save or edit a bit, validate it, and retrieve it later using structured results.
- Adding a shelf or bit changes completion candidates without regenerating completion scripts.
- A permanent shelf and every root-level `SHELF.md` survive all pruning operations.
- Verbatim prompt bodies survive ingestion and retrieval without modification.
- Invalid metadata does not prevent raw content access, and errors explain how to fix the issue.
- A release can be installed and upgraded through the documented supported routes.

## Deferred work

- QMD or another semantic search integration. Keep Markdown authoritative and indexes rebuildable; plain-text search is sufficient initially.
- Custom agent servers, MCP tools, and harness-specific automatic context injection.
- Automatic memory capture, background services, or hosted accounts.
- General-purpose schema languages, deeply nested shelf organization, and inherited guidance trees.
- A full terminal editor, synchronization service, or app-managed Git workflow.
- Self-updating binaries, a broad platform matrix, and advanced release attestations.

## References

- Original project concept: [idea.md](idea.md).
- [Usage](https://github.com/jdx/usage) and its [Rust quickstart](https://usage.jdx.dev/rust/quickstart#docs-and-manpages).
- [Usage runtime completions](https://usage.jdx.dev/rust/completions).
- [Demand](https://github.com/jdx/demand).
- [Agent Skills specification](https://agentskills.io/specification) and [Skills installer](https://github.com/vercel-labs/skills).
- [Homebrew taps](https://docs.brew.sh/Taps).
- [Usage docs deployment example](https://github.com/jdx/usage/blob/main/.github/workflows/docs.yml).
- [Usage release workflow example](https://github.com/jdx/usage/blob/main/.github/workflows/publish-cli.yml).
- Potential later retrieval integration: [QMD](https://github.com/tobi/qmd).
