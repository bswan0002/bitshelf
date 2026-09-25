# Install

> bitshelf is a prototype, and installation is for development and testing with disposable data. No stable release has been published yet.

## From source (recommended)

Install stable Rust with [rustup](https://rustup.rs), then in a bitshelf checkout:

```sh
cargo install --path . --locked
bs completion install        # optional; previews changes and asks first
bs init --store ~/bitshelf
```

Start a new shell to activate completion. To upgrade, update the checkout and rerun `cargo install`. The executable is `bs`, not `bitshelf`, and running it doesn't need Node.js or the Usage CLI. See [Shell completion](concepts/completion.md) for shell overrides, dry runs, and uninstalling.

## Agent skill

The skill is installed separately from the executable:

```sh
npx skills add bswan0002/bitshelf --skill bitshelf --global   # omit --global for project scope
npx skills update
```

Or copy `skills/bitshelf/` into a compatible agent's skill directory. See [Using with agents](agents.md).

## Release builds (when published)

Release automation exists but hasn't shipped a stable release. Once one is published:

**Homebrew** (personal tap):

```sh
brew install bswan0002/tap/bitshelf && bs completion install
brew upgrade bitshelf
```

The formula installs `bs`, a man page, and bash/zsh/fish completions, plus the skill under Homebrew's package share directory. `bs completion install` is optional if your shell already loads Homebrew's completions, and uninstalling it doesn't remove Homebrew's files.

**Archives.** Download the archive for your platform from GitHub Releases, verify it against `SHA256SUMS` (`sha256sum` on Linux, `shasum -a 256` on macOS), extract it, and copy `bs` onto your `PATH`. Archives also contain the man page, completions, and skill. To update, replace them with a verified newer release; there's no self-update.

Targets:

- macOS Apple Silicon (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Linux x86-64 (`x86_64-unknown-linux-gnu`, built on Ubuntu 22.04; requires a compatible glibc)

Prototype macOS archives are **unsigned and unnotarized**, so Gatekeeper may block browser downloads. Windows and Linux ARM are deferred.
