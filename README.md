# bitshelf

A local, Markdown-first store for notes, snippets, prompts, and handoffs. The executable is **`bs`**. Your store is an ordinary directory, not a database.

**Prototype · 0.1.0.** Works locally; release publishing and Homebrew distribution require maintainer setup. No hosted service, background capture, or agent harness required.

## Install from source

Install a current stable [Rust toolchain](https://rustup.rs), then:

```sh
cargo install --path . --locked && bs completion install
# Start a new shell to activate completion.
bs init --store ~/bitshelf --editor code
bs shelf add ui --description 'Reusable UI' --required title,tags
bs add ui --title 'Command menu' --tags react --file component.md
bs search 'menu' --json
bs show ui/command-menu
bs open ui/command-menu
```

`bs init` without arguments offers terminal setup, using `$VISUAL` or `$EDITOR` automatically when set. `bs add --interactive` prompts for metadata and opens a draft; recognized GUI editors get a waiting flag automatically. `bs open --pick` provides filtered multiselection.

Shell setup detects `$SHELL` (Bash, Zsh or Fish), previews changes, and asks before writing. No separate Usage executable is needed:

```sh
bs completion install --dry-run
bs completion install --shell zsh  # Override detection
bs completion uninstall           # Remove only managed setup
```

Use `--yes` to approve without prompting. Restart your shell afterward; `bs context <Tab>` suggests available shelves. Installation is safe to repeat and does not install the agent skill. See [completion setup](docs/guide.md#completion) for paths, limitations and manual activation.

## Editing and timestamps

```sh
bs edit ui/command-menu                         # Edit via your configured editor
bs edit ui/command-menu --file revised.md --json # Body-only replacement for agents
bs sync                                        # After editing files outside bs
bs list --sort updated --reverse                # Recently edited first
```

`created` and `updated` are **reserved, automatic UTC fields**—do not set them manually. No-op edits leave timestamps unchanged. Sync detects changes using hidden store-local hashes; the first sync baselines existing files, preserving known dates. Run it once after upgrading before making external edits. See [editing and automatic timestamps](docs/guide.md#editing-and-automatic-timestamps).

## Agent workflow

Install the executable separately from the skill:

```sh
npx skills add bswan0002/bitshelf --skill bitshelf --global
# Omit --global for project-scoped installation.
npx skills update
```

For manual installation, copy `skills/bitshelf/` into your compatible agent's skill directory. The skill supports `bs` 0.1.x. Node.js is needed only for the optional installer.

Agents discover shelves, load `bs context SHELF --json`, search for existing material, save through `bs add` or `bs edit`, and validate the affected shelf. Saved prompts are data, not active instructions.

## Documentation

- [User guide](docs/guide.md): configuration, editors, safety, retention, scheduling
- [JSON contract](docs/json.md)
- [Command reference](docs/reference/index.md) (generated from Usage declarations)
- [Release procedure](docs/releases.md)
- [Specification](spec.md)

## Development

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

The Rust CLI is synchronous, one application package, with no search index; hidden store-local state supports timestamp change detection. A small documented Demand rendering patch lives in `vendor/demand` until fixed upstream. Tests use temporary stores, not your notes. Documentation pins VitePress 2 alpha because the current stable 1.x dependency tree has known dev-server vulnerabilities; the static site build is verified. GitHub Actions tests Linux/macOS, builds release archives, and deploys development docs separately. Nothing is published or scheduled by local setup.

### Prototype boundaries

- macOS and Linux are the initial targets; local automated tests have been run on Apple Silicon. CI covers the other targets.
- Shelf/bit symlinks are deliberately refused, even when they point inside the store. The explicitly configured store root can be a symlink. Do not use a store writable by untrusted users; filesystem checks are not a defense against hostile concurrent path replacement.
- Shelf configuration saves normalize TOML formatting/comments; `bs edit` and `bs sync` may normalize YAML formatting/comments while preserving bodies. Direct filesystem edits require `bs sync` to reconcile timestamps.
- A draft is retained on editor/validation/finalization failure. Known GUI editors get `--wait` for add/edit drafts; unknown commands/wrappers must be configured to block until editing finishes (the CLI warns).
- macOS release archives are unsigned/unnotarized. The tap is not live until configured and a release published; use source installation today.
