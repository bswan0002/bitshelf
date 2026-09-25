# Releasing

Maintainer procedure for publishing bitshelf. User-facing installation lives in [docs/install.md](docs/install.md).

## One-time setup

1. Enable GitHub Pages with **GitHub Actions** as source. The docs workflow publishes `main` as clearly labeled development documentation.
2. Create `bswan0002/homebrew-tap` (or another personal tap with a compatible URL/install command). Set repository variable `HOMEBREW_TAP` to its full owner/repo and secret `HOMEBREW_TAP_TOKEN` to a narrowly scoped token with contents-write access to that tap. Without the variable, the tap update job is skipped. Update URLs in docs and `scripts/homebrew-formula.py` if publishing under another owner.

## Each release

1. Update `Cargo.toml`, `Cargo.lock`, the Usage version attribute in `src/cli/mod.rs`, `CHANGELOG.md`, and the skill's compatibility statement if necessary. Use a stable `vMAJOR.MINOR.PATCH` tag for Homebrew publishing.
2. Run `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, and the docs build. Manually test Demand terminal cancellation, draft failure recovery, and shell activation on release platforms.
3. Build the executable and regenerate reference/man/completions using `bash scripts/generate-docs.sh` with Usage CLI **6.11.1**. Commit generated `docs/reference` pages, not `dist`.
4. Push an annotated `v*` tag matching the package version. This is the explicit publication trigger. The release workflow creates a draft, tests/builds all targets, generates docs/completions, uploads archives and checksums, then publishes only after all builds succeed. It does not push tags for you.
5. Inspect the published release and optional tap job. Test `brew install`, `brew test`, and upgrade on supported platforms. A tap failure after publication requires rerunning that job; publication is not rolled back automatically.

The formula generator consumes checksums, never placeholder hashes:

```sh
python3 scripts/homebrew-formula.py v0.1.0 SHA256SUMS > bitshelf.rb
```

The workflow does not perform signing, notarization, Packslip attestations, or crates.io publishing.

## macOS signing decision

Prototype archives are unsigned and unnotarized. Source installation is the recommended prototype route; do not advertise frictionless downloaded-app installation. Signing/notarization must be configured before changing this claim.
