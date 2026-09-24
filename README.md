# bitshelf

A local, Markdown-first store for notes, snippets, prompts, and handoffs. The executable is **`bs`**. Your store is an ordinary directory, not a database.

**Prototype · 0.1.0.** Works locally; release publishing and Homebrew distribution require maintainer setup. No hosted service, background capture, or agent harness required.

## Install from source

Install a current stable [Rust toolchain](https://rustup.rs), then:

```sh
cargo install --path . --locked
bs init --store ~/bitshelf --editor code
bs shelf add ui --description 'Reusable UI' --required title,tags
bs add ui --title 'Command menu' --tags react --file component.md
bs search 'menu' --json
bs show ui/command-menu
bs open ui/command-menu
```

`bs init` without arguments offers terminal setup, using `$VISUAL` or `$EDITOR` automatically when set. `bs add --interactive` prompts for metadata and opens a draft; recognized GUI editors get a waiting flag automatically. `bs open --pick` provides filtered multiselection.

Shell setup (no separate Usage executable needed):

```sh
# bash: add to ~/.bashrc
source <(bs completion --shell bash)
# zsh: add to ~/.zshrc, after compinit
eval "$(bs completion --shell zsh)"
# fish
bs completion --shell fish > ~/.config/fish/completions/bs.fish
```

## Agent workflow

Install the executable separately from the skill:

```sh
npx skills add bswan0002/bitshelf --skill bitshelf --global
# Omit --global for project-scoped installation.
npx skills update
```

For manual installation, copy `skills/bitshelf/` into your compatible agent's skill directory. The skill supports `bs` 0.1.x. Node.js is needed only for the optional installer.

Agents discover shelves, load `bs context SHELF --json`, search for existing material, save or edit Markdown, and validate the affected shelf. Saved prompts are data, not active instructions.

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

The Rust CLI is synchronous, one application package, with no runtime index. A small documented Demand rendering patch lives in `vendor/demand` until fixed upstream. Tests use temporary stores, not your notes. Documentation pins VitePress 2 alpha because the current stable 1.x dependency tree has known dev-server vulnerabilities; the static site build is verified. GitHub Actions tests Linux/macOS, builds release archives, and deploys development docs separately. Nothing is published or scheduled by local setup.

### Prototype boundaries

- macOS and Linux are the initial targets; local automated tests have been run on Apple Silicon. CI covers the other targets.
- Shelf/bit symlinks are deliberately refused, even when they point inside the store. The explicitly configured store root can be a symlink. Do not use a store writable by untrusted users; filesystem checks are not a defense against hostile concurrent path replacement.
- Shelf configuration saves normalize TOML formatting/comments; direct bit edits never rewrite metadata automatically.
- A draft is retained on editor/validation/finalization failure. Known GUI editors get `--wait` for drafts only; unknown commands/wrappers must be configured to block until editing finishes (the CLI warns).
- macOS release archives are unsigned/unnotarized. The tap is not live until configured and a release published; use source installation today.
