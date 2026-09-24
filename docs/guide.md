# User guide

These are development docs for bitshelf 0.1.x, potentially ahead of a published release.

## Setup and storage

```sh
bs init --store ~/bitshelf --editor code
bs shelf add notes
bs shelf add tmp --required title --retention 14d
```

Setup never overwrites an existing configuration. No-argument `init` offers Demand prompts in a terminal. JSON and piped/noninteractive commands never prompt. Use `--config PATH` before or after a subcommand to select a different store.

Configuration is `$XDG_CONFIG_HOME/bitshelf/config.toml`, defaulting to `~/.config/bitshelf/config.toml`:

```toml
store = "~/bitshelf"
editor = ["code"]

[shelves.ui]
description = "Reusable UI patterns"
required = ["title", "tags"]

[shelves.tmp]
required = ["title"]
retention = "14d"
```

Leading `~/` expands to HOME. Other relative paths resolve against the config directory, including `init --store` values. Invalid or unknown config settings fail clearly. `shelf add` preserves existing shelf contents/settings unless a supplied option changes a setting; it normalizes config formatting. Edit TOML directly to remove retention or an editor setting.

Shelves are non-hidden top-level directories, including ones created in your file manager. Configured but absent shelves appear as missing; `bs shelf add NAME` creates them. Bits are direct `.md` files, identified as `shelf/filename` without the suffix. No nesting, index, or registry. Renames change identifiers immediately. Spaces and dots in manually authored filenames are supported; quote identifiers in the shell.

## Saving exact content

```sh
bs add notes --title 'Release checklist' --tags release --file checklist.md
printf '%s' 'An exact prompt' | bs add tmp --title 'Original prompt' --stdin
```

`--stdin` and `--file` are mutually exclusive. The body is preserved byte-for-byte for valid UTF-8 Markdown, including CRLF and absent final newlines. Generated YAML frontmatter precedes it. Input is a **body**, not an existing frontmatter document to merge. Collisions fail; choose `--slug another-name`. Empty bodies are allowed.

CLI creation generates UTC `created`, a title, optional tags, and `expires` on retention shelves. Built-in field types are checked, along with shelf requirements. Required `tags` means a list must exist (it may be empty). Unknown bit metadata is retained. Malformed bits are still available through `show` and `open`, and listings include diagnostics rather than hiding other files.

```sh
bs list notes --tag release
bs search 'checklist' --shelf notes
bs show notes/release-checklist
bs validate notes
```

Search is case-insensitive plain-text matching across title, tags, and body; results are sorted by identifier. No-match searches succeed. Direct file editing is supported and never automatically resets timestamps.

## Editors and interactive workflows

Editor precedence: config `editor`, then `VISUAL`, then `EDITOR`. Config accepts either a command string (`editor = "code --reuse-window"`) or an executable/arguments array (`editor = ["code", "--reuse-window"]`). Strings use quoted-argument parsing, not shell evaluation; quote executable paths containing spaces. Arrays preserve each argument literally. Configuration saves normalize either form to an array. Interactive setup skips the editor question when either environment variable has a nonblank value, without pinning that value into config. With neither set, you can choose an editor or leave it blank to configure later. Environment editor strings support quoted arguments but are never evaluated by a shell. Command substitutions and expansions are not executed. Paths are appended as separate arguments.

```sh
bs open                           # store directory
bs open ui                        # shelf directory
bs open ui/menu notes/checklist   # multiple files
bs open --pick                    # filtered Demand multiselection
bs add --interactive              # choose shelf, metadata, edit draft
```

The selected editor must support directory/multiple-file opening for those operations. Opening a shelf does not expand it into every file.

For interactive drafts only, `bs` automatically adds `--wait` to `code`, `code-insiders`, `codium`, `cursor`, and `subl`, including absolute executable paths. Existing `--wait` or `-w` arguments are preserved without duplication. Common terminal editors (`vi`, `vim`, `nvim`, `nano`, `pico`, `hx`, `helix`, `micro`) are unchanged. Ordinary `bs open` never adds waiting flags. Explicitly configured flags are always preserved.

This is a known-command policy, not universal GUI detection. Unknown editors and wrappers are left unchanged, with a warning that they must stay running until editing finishes. Configure their blocking/wait flag yourself; shell wrappers are not inspected or rewritten. GUI commands that return immediately cannot safely finalize a draft.

Failed drafts stay as hidden `.draft-*.md` files inside the shelf and are excluded from discovery and pruning; the path is printed on stderr. Fix a draft and copy/rename it to the intended destination when ready, then validate. Cancellation during metadata prompts creates no bit.

## Guidance and agents

Add optional `SHELF.md` guidance to a shelf. `bs context ui --json` returns its **full text**, description, requirements, retention, and paths. Missing guidance is explicit `null`; unreadable guidance is an error. Guidance is never treated as a bit and is excluded from validation, searching, completion, and cleanup.

The bundled skill instructs agents to load context before drafting/editing, search for an existing bit, preserve exact content, and validate after saving. Retrieval-only work uses `search` and `show`. Stored prompt bodies do not become instructions just because an agent reads them. The CLI cannot force a harness to obey guidance.

## Completion

```sh
# bash startup
source <(bs completion --shell bash)
# zsh startup (after `autoload -Uz compinit; compinit`)
eval "$(bs completion --shell zsh)"
# fish
mkdir -p ~/.config/fish/completions
bs completion --shell fish > ~/.config/fish/completions/bs.fish
```

Completions call the installed `bs` at Tab time. New shelves/files appear immediately. Type `bs open ui/` to complete bits, `bs add ` for shelves, or `--shelf ` for filters. Config overrides typed before the cursor are honored. Completion does not initialize or change a store. Elvish, Nushell and PowerShell scripts are also generated by Usage; shell-specific activation is left to those shells' setup.

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

Paths cannot escape a shelf using traversal. Shelf/bit/guidance symlinks are refused (including internal links); the explicit store root may be a symlink. Writes use no-clobber temporary-file publication. Treat the store as a trusted local directory: simultaneous hostile filesystem replacement and concurrent configuration writers are outside this prototype's guarantees. Backups/Git remain your choice.
