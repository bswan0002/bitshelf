# Install

## Homebrew

With [Homebrew](https://brew.sh) installed, use the personal tap:

```sh
brew install bswan0002/tap/bitshelf && bs completion install
```

The executable is `bs`. The formula includes a man page, bash/zsh/fish completions,
and matching docs and agent skill. Running `bs` needs neither Rust nor Node.js.
See [GitHub Releases](https://github.com/bswan0002/bitshelf/releases) for available versions.

`bs completion install` previews changes and asks before editing your shell
startup file; start a new shell afterward. See [Shell completion](concepts/completion.md)
for shell overrides, dry runs, and uninstalling.

Create your store:

```sh
bs init --store ~/bitshelf
```

To upgrade, back up your store, read the changelog, then run:

```sh
brew update
brew upgrade bitshelf
```

Update your agent's skill alongside the executable; see [release matching](releases.md).

## Agent skill

With Node.js and npm installed, run:

```sh
npx skills add bswan0002/bitshelf --skill bitshelf --global
```

Choose your agents when prompted. `--global` makes the skill available across
repositories. See [using agents](agents.md) and [matching releases](releases.md)
for installing the skill from a specific release tag. The skill installer does
not install the `bs` executable.

## From source

For development or when a downloaded macOS binary is blocked, install
[rustup](https://rustup.rs) and build a reviewed release tag. The checkout selects
the pinned toolchain; see [platforms](platforms.md) for the minimum source toolchain.

```sh
git clone --branch v0.1.0 https://github.com/bswan0002/bitshelf.git
cd bitshelf
cargo install --path . --locked
bs completion install        # optional; previews changes and asks first
```

Use the skill and docs from that same checkout. To upgrade a source installation,
check out the intended release tag and rerun `cargo install --path . --locked`.

## Release archives

Download the archive for your platform from GitHub Releases, verify it against `SHA256SUMS` (`sha256sum` on Linux, `shasum -a 256` on macOS), extract it, and copy `bs` onto your `PATH`. Archives also contain the man page, completions, matching Markdown docs and skill, third-party notices, dependency inventory and BUILD-INFO.json. To update, replace them with a verified newer release; there's no self-update.

Targets:

- macOS Apple Silicon (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Linux x86-64 (`x86_64-unknown-linux-gnu`, built and tested by the release workflow on Ubuntu 22.04; glibc 2.35 baseline; see [platform support](platforms.md) for which platforms have executed evidence)

Initial 0.1.x macOS archives are **not Developer ID signed or notarized**, so Gatekeeper may block browser downloads. Windows and Linux ARM are deferred.

Exact tooling, runtime minimums, per-platform evidence gates and BUILD-INFO.json are described in [platform support](platforms.md).

The first-release [signing/provenance policy](signing.md) explicitly defers Developer
ID signing, notarization and independent attestations. A local ad-hoc Mach-O
signature is not trusted publisher identity; Gatekeeper assessment rejected the
local candidate. A quarantined browser download has not been tested. Source installation from a reviewed tag is the fallback. No
Gatekeeper/quarantine bypass is recommended.
