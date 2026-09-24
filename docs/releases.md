# Installation and releases

## Available now: source installation

Install stable Rust and run `cargo install --path . --locked` in this checkout. Repeat after updating the checkout to upgrade. The executable is `bs`, not `bitshelf`. No Node.js or Usage CLI is needed to run it.

Binary releases and a personal Homebrew tap are **prepared but not provisioned by this prototype**. Do not assume the following release routes are live until the maintainer publishes a tag and configures the tap.

## Prepared release routes

After release provisioning:

```sh
brew install bswan0002/tap/bitshelf
bs --version
brew upgrade bitshelf
```

The formula installs `bs`, a man page, and bash/zsh/fish completions. Users activate their shell's completion system separately.

Alternatively download the archive matching your platform from GitHub Releases, verify it against `SHA256SUMS` (`sha256sum` on Linux or `shasum -a 256` on macOS), extract it, and copy `bs` to a directory on PATH. Archives also contain the man page, completions, and skill. Updating means replacing those files with a verified newer release. There is no self-update command.

Initial targets:

- macOS Apple Silicon (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Linux x86-64 (`x86_64-unknown-linux-gnu`, built on Ubuntu 22.04; requires compatible glibc)

**macOS signing decision:** prototype archives are unsigned and unnotarized. Browser downloads may be blocked by Gatekeeper. Source installation is the recommended prototype route; do not advertise frictionless downloaded-app installation. Signing/notarization must be configured before changing this claim. Windows and Linux ARM distribution are deferred.

## Maintainer checklist

1. Enable GitHub Pages with **GitHub Actions** as source. The docs workflow publishes main as clearly labeled development documentation.
2. Create `bswan0002/homebrew-tap` (or another personal tap with a compatible URL/install command). Set repository variable `HOMEBREW_TAP` to its full owner/repo and secret `HOMEBREW_TAP_TOKEN` to a narrowly scoped token with contents-write access to that tap. Without the variable, the tap update job is skipped. Update URLs in docs and `scripts/homebrew-formula.py` if publishing under another owner.
3. Update `Cargo.toml`, `Cargo.lock`, the Usage version attribute in `src/cli/mod.rs`, `CHANGELOG.md`, and the skill's compatibility statement if necessary. Use a stable `vMAJOR.MINOR.PATCH` tag for Homebrew publishing.
4. Run `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, and the docs build. Manually test Demand terminal cancellation, draft failure recovery, and shell activation on release platforms.
5. Build the executable and regenerate reference/man/completions using `bash scripts/generate-docs.sh` with Usage CLI **6.11.1**. Commit generated `docs/reference` pages, not `dist`.
6. Push an annotated `v*` tag matching the package version. This is the explicit publication trigger. The release workflow creates a draft, tests/builds all targets, generates docs/completions, uploads archives and checksums, then publishes only after all builds succeed. It does not push tags for you.
7. Inspect the published release and optional tap job. Test `brew install`, `brew test`, and upgrade on supported platforms. A tap failure after publication requires rerunning that job; publication is not rolled back automatically.

The formula generator consumes checksums, never placeholder hashes:

```sh
python3 scripts/homebrew-formula.py v0.1.0 SHA256SUMS > bitshelf.rb
```

Release and Pages jobs have not been executed merely by building the local prototype. CI credentials, repository settings, runner availability, and cross-platform install behavior must be verified in GitHub. The workflow does not perform signing, notarization, Packslip attestations, or crates.io publishing.

## Agent skill installation

Executable installation and skill installation are separate:

```sh
npx skills add bswan0002/bitshelf --skill bitshelf --global
# Project scoped: omit --global
npx skills update
```

Or copy `skills/bitshelf` into a compatible agent's skill directory manually. The skill supports CLI 0.1.x and relies on `bs --help` instead of duplicating the command reference. Node.js is required only for the optional `npx` path. Skill files are also included in release archives and installed under Homebrew's package share directory.
