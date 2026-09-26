# Install

> bitshelf is a prototype, and installation is for development and testing with disposable data. No stable release has been published yet.

## From source (recommended)

Install the pinned Rust toolchain (rust-toolchain.toml; see [platforms](platforms.md)) with [rustup](https://rustup.rs), then in a bitshelf checkout:

```sh
cargo install --path . --locked
bs completion install        # optional; previews changes and asks first
bs init --store ~/bitshelf
```

Start a new shell to activate completion. To upgrade, update the checkout and rerun `cargo install`. The executable is `bs`, not `bitshelf`, and running it doesn't need Node.js or the Usage CLI. See [Shell completion](concepts/completion.md) for shell overrides, dry runs, and uninstalling.

## Agent skill

Copy `skills/bitshelf/` from the same checkout or release archive as the executable
into your agent's skill directory. With Homebrew, the matching source is
`$(brew --prefix bitshelf)/share/bitshelf/skills/bitshelf/`. The bundled VERSION
file identifies the CLI release. See [matching releases](releases.md) and
[using agents](agents.md). Avoid updating the skill from main independently of bs.

## Release builds (when published)

Release automation exists but hasn't shipped a stable release. Once one is published:

**Homebrew** (personal tap):

```sh
brew install bswan0002/tap/bitshelf && bs completion install
brew upgrade bitshelf
```

The formula installs `bs`, a man page, and bash/zsh/fish completions, plus the skill under Homebrew's package share directory. `bs completion install` is optional if your shell already loads Homebrew's completions, and uninstalling it doesn't remove Homebrew's files.

**Archives.** Download the archive for your platform from GitHub Releases, verify it against `SHA256SUMS` (`sha256sum` on Linux, `shasum -a 256` on macOS), extract it, and copy `bs` onto your `PATH`. Archives also contain the man page, completions, matching Markdown docs and skill, third-party notices, dependency inventory and BUILD-INFO.json. To update, replace them with a verified newer release; there's no self-update.

Targets:

- macOS Apple Silicon (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Linux x86-64 (`x86_64-unknown-linux-gnu`, built/tested on Ubuntu 22.04; glibc 2.35 baseline)

Prototype macOS archives are **not Developer ID signed or notarized**, so Gatekeeper may block browser downloads. Windows and Linux ARM are deferred.

Exact tooling, runtime minimums, per-platform evidence gates and BUILD-INFO.json are described in [platform support](platforms.md).

The first-release [signing/provenance policy](signing.md) explicitly defers Developer
ID signing, notarization and independent attestations. A local ad-hoc Mach-O
signature is not trusted publisher identity; Gatekeeper assessment rejected the
local candidate. Source installation from a reviewed tag is the fallback. No
Gatekeeper/quarantine bypass is recommended.
