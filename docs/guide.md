# User guide

These are development docs for bitshelf 0.1.x, potentially ahead of a published release.

## Setup and storage

```sh
bs init --store ~/bitshelf --editor code
bs shelf add notes
bs shelf add tmp --retention 14d
```

Setup never overwrites an existing configuration. No-argument `init` offers Demand prompts in a terminal. JSON and piped/noninteractive commands never prompt. Use `--config PATH` before or after a subcommand to select a different store.

Configuration is `$XDG_CONFIG_HOME/bitshelf/config.toml`, defaulting to `~/.config/bitshelf/config.toml`:

```toml
store = "~/bitshelf"
editor = ["code"]

```

Each shelf owns its settings in `<store>/<shelf>/bs.toml`, for example:

```toml
description = "Reusable UI patterns"
required = ["tags"]
# retention = "14d" # Optional; omit for permanent storage
```

### Namespaced tag rules

Configure tag namespaces in a shelf's `bs.toml`:

```toml
[tag_rules.project]
required = true
allowed = ["bitshelf", "switchboard", "data-explorer"]
```

Bits still use ordinary string tags, for example:

```yaml
tags: ["project:bitshelf", "rust", "kind:guide"]
```

`required = true` requires at least one tag in that namespace; it defaults to false. Every tag in a configured namespace must have an allowed value. Multiple allowed project tags are fine. Ordinary tags and unconfigured namespaces remain unrestricted. Matching is exact and case-sensitive, with no automatic normalization. An empty value such as `project:` is invalid for a configured namespace.

`allowed` is mandatory and must be nonempty, with no duplicates. Namespace names and allowed values must be nonempty and contain no whitespace, control characters, colons, or commas. Unknown rule settings are rejected. The existing `required = ["tags"]` checks field presence only; namespace requirements enforce membership independently.

Use existing flags, such as `bs add docs/setup --tags project:bitshelf,rust --file setup.md` and `bs list docs --tag project:bitshelf`. Add/edit reject invalid metadata; directly edited invalid bits remain readable and `bs validate` reports violations. Changing rules does not rewrite bits. `bs context docs` (including `--json`) exposes the complete `tag_rules` so authors can discover requirements before writing. Edit shelf TOML directly to manage these rules.

### Configuration paths

Leading `~/` expands to HOME. Other relative paths resolve against the global config directory, including `init --store` values. Invalid or unknown config settings fail clearly. `shelf add` creates `bits/` and writes `bs.toml`, preserving existing contents/settings unless a supplied option changes a setting. Edit shelf TOML directly to remove retention; edit global TOML to remove an editor setting. Missing `bs.toml` uses default shelf settings.

```text
~/bitshelf/ui/
├── bs.toml
├── SHELF.md            # optional guidance
├── bits/
│   └── command-menu.md
└── scripts/            # optional, ordinary shelf-local files
    └── import.py
```

Shelves are non-hidden top-level directories containing `bits/` or `bs.toml`, including ones created in your file manager. A shelf with `bs.toml` but no `bits/` is reported as missing its bits directory; `bs shelf add NAME` repairs it. Removing a whole shelf removes it from discovery; there is no global registry. In shelf-list JSON, `configured` means `bs.toml` exists and `missing` means `bits/` is absent or not a directory.

Bits are direct, non-hidden `.md` files inside `bits/`, identified as `shelf/filename` without the suffix or `bits/` component. Everything else in the shelf is ignored and preserved by content operations. No nested bits or search index; internal `.bitshelf/` state tracks content hashes for timestamp reconciliation, not bit discovery. Renames change identifiers immediately. Spaces and dots in bit names are supported; quote identifiers in the shell. An ID is exactly `shelf/bit-name`, not a separate UUID. Supply it directly to `add`; names are not generated from titles.

## Saving exact content

```sh
bs add notes/release-checklist --tags release --file checklist.md
printf '%s' 'An exact prompt' | bs add tmp/original-prompt --file -
```

`--file -` reads stdin, as does `--stdin`; the two flags are mutually exclusive. The body is preserved byte-for-byte for valid UTF-8 Markdown, including CRLF and absent final newlines. Generated YAML frontmatter precedes it. Input is a **body**, not an existing frontmatter document to merge. Collisions fail; choose a different ID. Empty bodies are allowed.

Title metadata is optional unless the shelf explicitly sets `required = ["title"]`. Use `--title TEXT` for an extra description; supplied titles must be nonempty. IDs are the default display names, and title changes never rename a bit.

**`created` and `updated` are reserved, automatically managed timestamps.** Generated dates use UTC; valid imported dates retain their representation. Users and agents must not supply or manually edit them. CLI creation sets both to the creation time, along with optional title/tags metadata, and `expires` on retention shelves. Built-in field types are checked, along with shelf requirements. Required `tags` means a list must exist (it may be empty). Unknown bit metadata is retained. Malformed bits are still available through `show` and `open`, and listings include diagnostics rather than hiding other files.

```sh
bs list notes --tag release
bs search 'checklist' --shelf notes
bs show notes/release-checklist
bs validate notes
```

Search is case-insensitive plain-text matching across ID, optional title, tags, and body; results are sorted by identifier. No-match searches succeed. Direct file editing is supported; run `bs sync` afterward to reconcile timestamps. Read-only commands never reset timestamps.

### Output and pipelines

`list` and `search` print one ID per line by default:

- `--long` prints `ID<TAB>title`; missing titles leave an empty title column.
- `--paths` prints absolute filesystem paths instead of IDs.
- `--null` terminates each ID/path with NUL (including the last), for tools such as `xargs -0`.
- `--json` emits rich structured results. It cannot be combined with these text-output flags.

`--long` cannot be combined with `--paths` or `--null`. Empty text results produce no output.
Use `--null` for safe ID/path composition, and JSON rather than parsing descriptive titles.

`show` prints the entire unmodified file. `show --body` removes frontmatter and preserves the
body exactly, including line endings and absent final newlines. A file without frontmatter
is all body; malformed frontmatter causes `--body` to fail rather than guess. With `--json`,
the same selection is returned in `content`.

```sh
bs list notes --paths --null | xargs -0 rg 'pattern'
bs show notes/release-checklist --body | bs add notes/checklist-copy --file -
printf '%s' 'Replacement body' | bs edit notes/checklist-copy --file -
```

Closed output pipes (for example, a consumer exiting early) terminate quietly with status 0.
Other output errors remain failures. Results are still collected in memory; these flags do
not introduce incremental processing.

## Editors and interactive workflows

Editor precedence: config `editor`, then `VISUAL`, then `EDITOR`. Config accepts either a command string (`editor = "code --reuse-window"`) or an executable/arguments array (`editor = ["code", "--reuse-window"]`). Strings use quoted-argument parsing, not shell evaluation; quote executable paths containing spaces. Arrays preserve each argument literally. Configuration saves normalize either form to an array. Interactive setup skips the editor question when either environment variable has a nonblank value, without pinning that value into config. With neither set, you can choose an editor or leave it blank to configure later. Environment editor strings support quoted arguments but are never evaluated by a shell. Command substitutions and expansions are not executed. Paths are appended as separate arguments.

```sh
bs open                           # store directory
bs open ui                        # shelf directory
bs open ui/menu notes/checklist   # multiple files
bs open --pick                    # filtered Demand multiselection
bs add --interactive              # choose shelf, bit name, tags; edit draft
```

Interactive add does not ask for a title. Supply `--title` or add it in the draft if the shelf requires one. Passing an ID (`bs add notes/draft --interactive`) skips shelf/name selection.

The selected editor must support directory/multiple-file opening for those operations. Opening a shelf does not expand it into every file.

For interactive add drafts and `bs edit`, `bs` automatically adds `--wait` to `code`, `code-insiders`, `codium`, `cursor`, and `subl`, including absolute executable paths. Existing `--wait` or `-w` arguments are preserved without duplication. Common terminal editors (`vi`, `vim`, `nvim`, `nano`, `pico`, `hx`, `helix`, `micro`) are unchanged. Ordinary `bs open` never adds waiting flags. Explicitly configured flags are always preserved.

This is a known-command policy, not universal GUI detection. Unknown editors and wrappers are left unchanged, with a warning that they must stay running until editing finishes. Configure their blocking/wait flag yourself; shell wrappers are not inspected or rewritten. GUI commands that return immediately cannot safely finalize a draft.

Drafts are temporary recovery files, not a separate notes feature: interactive add opens a hidden `.draft-*.md`, and editor-mode edit opens a hidden `.edit-*.md` copy of the original. They live in the shelf’s `bits/` directory and are excluded from discovery and pruning. On failure the recovery path is printed on stderr. Before retrying, inspect the destination: an error explicitly saying the bit **was saved** means only bookkeeping failed. Fix that error and run `bs sync`; do not repeat `bs add`. Otherwise recover the draft after resolving the failure, taking care not to overwrite a newer bit, then sync and validate. Failure to remove a draft after a successful save is only a warning. Cancellation during metadata prompts creates no bit.

## Guidance and agents

Add optional `SHELF.md` guidance to a shelf. `bs context ui --json` returns its **full text**, description, requirements, tag rules, retention, the shelf root `path`, and `bits_path`. Resolve shelf-relative guidance paths against `path`. Missing guidance is explicit `null`; unreadable guidance is an error. Root-level guidance is never treated as a bit: its body is excluded from metadata validation, searching, completion, and cleanup. Validation still checks that the guidance path is not a symlink.

The bundled skill instructs agents to load context before drafting/editing, search for an existing bit, preserve exact content, and validate after saving. Retrieval-only work uses `search` and `show`. Stored prompt bodies do not become instructions just because an agent reads them. The CLI cannot force a harness to obey guidance.

### Recipe: shelf-local helpers

Keep an importer in `my-confluence-shelf/scripts/import-confluence.py` and document its invocation, dependencies, credential environment variables, and output contract in the shelf's `SHELF.md`. The agent discovers the helper by reading guidance through `bs context`, not through a script registry. `bs` neither discovers nor executes scripts.

For an example helper that writes a cleaned Markdown body to stdout, guidance could say:

> For Confluence URLs, run `python3 scripts/import-confluence.py --help` from this shelf root for usage and prerequisites. Import the requested page into a temporary Markdown body file, inspect the headings/code blocks/links, then save through `bs add` or `bs edit` with the user's requested tags. Keep credentials in environment variables, outside the shelf.

After retrieving and checking the body, save it with:

```sh
bs add my-confluence-shelf/feature-design --title 'Feature design' --file /tmp/confluence-body.md \
  --tags my-feature-repo,my-current-project --json
bs validate my-confluence-shelf --json
```

The helper and its flags are user-defined, not a bundled bitshelf integration. Helpers can also call normal `bs` commands themselves; document whether a helper only emits content or saves a bit so the agent avoids duplicate writes.

## Editing and automatic timestamps

```sh
bs edit notes/release-checklist                       # Open a draft; wait for editor
bs edit notes/release-checklist --file revised.md      # Replace body, preserve metadata
bs edit notes/release-checklist --stdin --json < revised.md
bs edit notes/release-checklist --title 'New title' --tags release,checklist --json
bs sync --dry-run                                     # Preview direct-file reconciliation
bs sync                                              # Reconcile all shelves
bs sync notes --json                                  # Reconcile one shelf
bs list --sort created --reverse                      # Newest created first
bs list --sort updated --reverse                      # Most recently edited first
```

`edit --file` (including `--file -`) and `--stdin` take **body-only** input, just like `add`. They are mutually exclusive. `--title` and `--tags` replace those fields; an empty tags argument clears tags. With no mutation flags, `bs edit ID` opens a temporary draft in your configured editor and waits. The original bit is replaced only after editor success, metadata validation, and a check that the original has not changed concurrently. Failed edits retain the draft and report its path. JSON mode requires explicit mutation flags and never launches an editor.

**Reserved fields:** `created` is immutable after tracking begins. `updated` advances automatically for a real content or user-metadata change. A no-op edit of a tracked, reconciled bit leaves timestamps unchanged; first-time tracking can fill missing dates, and an edit can detect a pending external change. CLI editing and sync preserve expiration and unknown metadata; neither extends retention. Manual changes to tracked `created`/`updated` fields are restored from the last known values. There are no flags for setting them. Timestamp rewrites may normalize YAML formatting/comments; the Markdown body is preserved verbatim. Frontmatter key order, comments, and reserved timestamp changes alone are not content changes.

**Direct editor saves:** `bs open` and external editors do not reconcile timestamps automatically. Run `bs sync` afterward. It compares SHA-256 hashes of the body and non-reserved metadata against hidden store-local `.bitshelf/state.json` bookkeeping. `updated` records **detection time**, not an inferred filesystem modification time. There is no background watcher. `list`, `show`, `search`, and `validate` remain read-only.

**Existing/imported files:** the first sync establishes a baseline. Valid existing timestamps are preserved; absent timestamps use the time the file is first tracked, not invented historical dates. Existing `created` is never inferred from filesystem birth time. Edits made before the baseline cannot be detected retroactively. Run `bs sync` before externally editing imported files. A previously unused identifier is baselined on discovery. State is keyed by identifier, not inode: editors that save by replacing the file retain timestamp history. Add/edit/sync/prune discard tracking records when saving state for paths observed missing, including deleted shelves; pruning also discards records for removed bits. Run sync after deleting or renaming files, before reusing their old identifiers. If a file is deleted and replaced at the same identifier between commands, that replacement is indistinguishable from an external edit and retains the identifier’s history. `bs add` always starts fresh history for its new bit. Losing `.bitshelf/state.json` does not lose notes or frontmatter dates, but loses change history; the next sync re-baselines. Include it in backups if you want to retain change detection, and do not edit it by hand.

Sync reconciles readable bits even when shelf-required fields such as title or tags are missing or user metadata is invalid. Use `bs validate` to check those authoring requirements; sync does not fill them in or change them. Malformed YAML, invalid untracked timestamps, and file read/write failures still produce per-bit errors; other readable bits are processed and sync exits nonzero if any processed bit fails. `--dry-run` writes neither bits nor tracking state. Add/edit/sync/prune use `.bitshelf/state.lock` to prevent competing CLI timestamp writes and pruning. An edit holds no lock while the editor is open: finalization acquires the lock, reloads current tracking state, and refuses to overwrite a concurrently changed original. After a crash, remove a stale lock only after confirming no such command is running. Bit updates and state updates are separate atomic writes, not a cross-file transaction; after an interrupted write, rerun sync (a pending edit may be timestamped at retry time). Sync does not delete notes or prune expiration.

List defaults to identifier order. Timestamp sorting is ascending unless `--reverse` is supplied, uses identifier order to break ties, and places missing/invalid dates last in either direction. Sorting does not implicitly sync.

## Completion

Install completion setup explicitly after installing the executable:

```sh
bs completion install                   # Detect from $SHELL; preview and confirm
bs completion install --shell zsh       # Override detection
bs completion install --dry-run         # Preview without writing or prompting
bs completion install --yes             # Approve without prompting (scripts/CI)
bs completion uninstall                 # Preview and confirm managed removal
```

Automatic setup supports Bash, Zsh and Fish. It targets one shell, not every installed shell. If `$SHELL` differs from the shell you are currently running, pass `--shell`.

- **Bash:** appends a managed block to `~/.bashrc`. Login shells must already source `.bashrc` from their profile; the installer does not edit profiles.
- **Zsh:** appends a managed block to `$ZDOTDIR/.zshrc` (otherwise `~/.zshrc`). The block initializes `compinit` only if `compdef` is not already available, then loads the completion script.
- **Fish:** writes a managed `bs.fish` under `$XDG_CONFIG_HOME/fish/completions` (otherwise `~/.config/fish/completions`). Rerun installation after upgrading to refresh this generated file.

Start a new shell afterward: an executable cannot update completion in its parent shell. `bs context <Tab>` should now suggest shelves; an fzf completion UI can still display the suggestions.

Repeated installation does not duplicate managed setup. Existing exact Bash/Zsh activation lines from the manual instructions below are recognized and left alone. Uninstall removes only managed blocks/files, not manual or package-manager setup. Symlinked dotfiles and configuration directories are supported: the preview shows the resolved target, changes update that target, and links remain intact. Uninstall leaves a symlinked Fish target empty rather than breaking its link. Broken/cyclic links, non-file targets, existing nonempty unmanaged Fish files, and malformed markers are rejected with an error rather than overwritten. Unrelated rc content and permissions are preserved (a missing final newline is added when appending). No bitshelf store or agent skill is initialized or installed. `--json` is not supported for completion commands.

For manual setup, script generation remains available:

```sh
# bash startup
source <(bs completion --shell bash)
# zsh startup (after `autoload -Uz compinit; compinit`)
eval "$(bs completion --shell zsh)"
# fish
mkdir -p ~/.config/fish/completions
bs completion --shell fish > ~/.config/fish/completions/bs.fish
```

Completions call the installed `bs` at Tab time. New shelves/files appear immediately. Type `bs open ui/` to complete bits, `bs add ` for shelf prefixes such as `ui/` (then type a new bit name), or `--shelf ` for filters. Config overrides typed before the cursor are honored. Completion does not initialize or change a store. Elvish, Nushell and PowerShell scripts are also generated by Usage; shell-specific activation is left to those shells' setup.

## Expiration and scheduling

Retention accepts positive whole days only (`14d`). New bits get an explicit expiration at creation + retention. Changing retention changes only future bits. Removing retention disables pruning for that shelf; permanent shelves are never pruned.

```sh
bs prune tmp --dry-run --json
bs prune tmp --json
bs prune --dry-run                 # all shelves, same rules
```

Only valid explicit timestamps at/before the current time are eligible. Missing/invalid expiration or invalid metadata is reported and skipped, with status 1. Guidance, hidden drafts, permanent shelves, symlinks, and future bits survive. No modification-time guesses. Back up your store if you need recovery; pruning removes files, not moves them to trash.

Example cron entry (replace paths; scheduler PATH/HOME may differ):

```cron
0 9 * * * /home/you/.cargo/bin/bs --config /home/you/.config/bitshelf/config.toml prune --json >> /home/you/bitshelf-prune.log 2>&1
```

macOS launchd: save a plist as `~/Library/LaunchAgents/local.bitshelf.prune.plist`, replacing all `/Users/you` values:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>local.bitshelf.prune</string>
  <key>ProgramArguments</key><array>
    <string>/Users/you/.cargo/bin/bs</string>
    <string>--config</string><string>/Users/you/.config/bitshelf/config.toml</string>
    <string>prune</string><string>--json</string>
  </array>
  <key>StartCalendarInterval</key><dict><key>Hour</key><integer>9</integer><key>Minute</key><integer>0</integer></dict>
  <key>StandardOutPath</key><string>/Users/you/Library/Logs/bitshelf-prune.log</string>
  <key>StandardErrorPath</key><string>/Users/you/Library/Logs/bitshelf-prune-errors.log</string>
</dict></plist>
```

Load with `launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/local.bitshelf.prune.plist`. Remove with `launchctl bootout gui/$(id -u) ~/Library/LaunchAgents/local.bitshelf.prune.plist`. Run a dry run manually first. bitshelf never installs a scheduler or daemon itself.

## Safety and limits

Paths cannot escape a shelf using traversal. Managed shelf paths (shelves, `bits/`, bit files, `bs.toml`, and `SHELF.md`) and tracking state refuse symlinks, including internal links; the explicit store root may be a symlink. Discovery warns and skips linked shelves/bits. Validation reports links as errors, including dangling links and non-hidden store-root links whose targets are not inspected. Auxiliary files such as `scripts/` are not traversed. New bits use no-clobber temporary-file publication; edits atomically replace the original after a conflict check. Treat the store as a trusted local directory: simultaneous hostile filesystem replacement and concurrent configuration writers are outside this prototype's guarantees. Backups/Git remain your choice.
