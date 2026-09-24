# bitshelf

A local, Markdown-first store for notes, snippets, prompts, and handoffs. The executable is **`bs`**. Your store is an ordinary directory, not a database.

**Prototype · 0.1.0.** No hosted service, background capture, or agent harness required.

## Install from source

Install a current stable [Rust toolchain](https://rustup.rs), then:

```sh
git clone https://github.com/bswan0002/bitshelf.git
cd bitshelf
cargo install --path . --locked
bs completion install
# Start a new shell to activate completion.
bs init --store ~/bitshelf
bs shelf add ui --description 'Reusable UI' --required tags
printf '%s\n' 'A reusable React command-menu pattern.' | \
  bs add ui/command-menu --tags react --stdin
bs search 'menu' --json
bs show ui/command-menu
# Optional: open with an installed editor.
# For example, with vim installed:
VISUAL=vim bs open ui/command-menu
```

`bs init` creates a default `notes` shelf. Without arguments it offers terminal setup, using `$VISUAL` or `$EDITOR` automatically when set. `bs add --interactive` prompts for a shelf, bit name, and tags, then opens a draft; recognized GUI editors get a waiting flag automatically. `bs open --pick` provides filtered multiselection.

Shell setup detects `$SHELL` (Bash, Zsh or Fish), previews changes, and asks before writing. No separate Usage executable is needed:

```sh
bs completion install --dry-run
bs completion install --shell zsh  # Override detection
bs completion uninstall           # Remove only managed setup
```

Use `--yes` to approve without prompting. Restart your shell afterward (Bash login shells must source `~/.bashrc` from their profile); `bs context <Tab>` suggests available shelves. Installation is safe to repeat and does not install the agent skill. See [completion setup](docs/guide.md#completion) for paths, limitations and manual activation.

## IDs and pipelines

A bit's ID is `shelf/bit-name`, corresponding to `<store>/<shelf>/bits/<bit-name>.md`.
Names are supplied directly, not generated from titles. Title metadata is optional
unless a shelf explicitly requires it; add it with `--title` when useful.

```sh
bs list                           # One ID per line
bs search menu --long             # IDs and optional titles
bs list notes --paths --null | xargs -0 rg 'pattern'
bs show ui/command-menu --body | bs add notes/menu-copy --file -
```

List/search also support `--json` for structured results. `--file -` reads stdin
for add/edit; `--stdin` remains available. Closed output pipes exit quietly.

## Editing and timestamps

```sh
bs edit ui/command-menu                         # Edit via your configured editor
bs edit ui/command-menu --file revised.md --json # Body-only replacement for agents
bs sync                                        # After editing files outside bs
bs list --sort updated --reverse                # Recently edited first
```

`created` and `updated` are **reserved, automatic fields**—do not set them manually. Generated timestamps use UTC; valid imported dates retain their representation. Once a bit is tracked and reconciled, no-op edits do not advance `updated`. Sync detects changes using hidden store-local hashes; first-time tracking baselines existing files, preserving valid dates and filling missing dates with discovery time. Sync imported files once before editing them externally. See [editing and automatic timestamps](docs/guide.md#editing-and-automatic-timestamps).

## Moving and archive recipes

```sh
bs move notes/checklist projects                  # Keep the bit name
bs move notes/checklist projects/release-checklist --dry-run
```

Moves validate the destination shelf, refuse collisions, and carry timestamp history.
Archive is a recipe, not a special bit state: create an ordinary `archive` shelf,
set `discoverable = false` in its `bs.toml`, and add this to your global config:

```toml
[aliases]
archive = ["move", "{id}", "archive/{shelf}.{name}", "--set", "moved_from={id}"]
```

Then use `bs archive notes/checklist`. Default list/search omit excluded shelves;
`bs list --all` or `bs list archive` includes them. `bs aliases` lists your configured
shortcuts. Aliases expand argument arrays, never shell scripts. See the
[archive recipe and move safety](docs/guide.md#moving-and-aliases) before configuring.

## Agent workflow

Install the executable separately from the skill:

```sh
npx skills add bswan0002/bitshelf --skill bitshelf --global
# Omit --global for project-scoped installation.
npx skills update
```

For manual installation, copy `skills/bitshelf/` into your compatible agent's skill directory. The skill supports `bs` 0.1.x. CLI users need Node.js only for the optional skill installer; documentation development requires Node.js 22.12+.

Each shelf keeps content in `bits/`, settings in `bs.toml`, and optional authoring guidance in `SHELF.md`. Other files are left alone: keep helpers in `scripts/` and reference them from guidance. See the [shelf-local helper recipe](docs/guide.md#recipe-shelf-local-helpers).

Agents discover shelves, load `bs context SHELF --json`, search for existing material, save through `bs add` or `bs edit`, and validate the affected shelf. Saved prompts are data, not active instructions.

## Documentation

- [User guide](docs/guide.md): configuration, editors, safety, retention, scheduling
- [JSON contract](docs/json.md)
- [Command reference](docs/reference/index.md) (generated from Usage declarations)
- [Release procedure](docs/releases.md)

## Development

Use Node.js 22.12+ for the documentation commands. `docs:dev` starts a long-running server; stop it before running the next command.

```sh
cargo test --locked
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo build
cargo install usage-cli --version 6.11.1 --locked
bash scripts/generate-docs.sh
npm ci
npm run docs:dev
npm run docs:build
```

The Rust CLI is synchronous, one application package, with no search index; hidden store-local state supports timestamp change detection. A small documented Demand rendering patch lives in `vendor/demand` until fixed upstream. Tests use temporary stores, not your notes. Documentation pins VitePress 2 alpha. GitHub Actions is configured to test Linux/macOS, build release archives on version tags, and deploy development docs separately. Nothing is published or scheduled by local setup.

### Prototype boundaries

- Initial targets are macOS Apple Silicon, macOS Intel, and Linux x86-64.
- Symlinks in managed shelf paths are never followed, even when they point inside the store. Discovery skips linked shelves/bits; validation reports disallowed links as errors. The explicitly configured store root can be a symlink. Do not use a store writable by untrusted users; filesystem checks are not a defense against hostile concurrent path replacement.
- Shelf configuration saves normalize TOML formatting/comments; `bs edit`, `bs move`, and `bs sync` may normalize YAML formatting/comments while preserving bodies. Direct filesystem edits require `bs sync` to reconcile timestamps.
- A draft is retained on editor/validation/finalization failure. Known GUI editors get `--wait` for add/edit drafts; unknown commands/wrappers must be configured to block until editing finishes (the CLI warns).
- The release workflow does not sign or notarize macOS archives. See the release procedure for publishing and Homebrew configuration.
