# Build and platform boundaries

Release/source tooling pins Rust **1.98.1**, Usage library/CLI **6.11.1**,
Python **3.12.9**, and Node **24.21.0** for
documentation. Cargo.lock/package-lock.json are committed. Runtime bs needs none
of Python, Node or Usage CLI; external helper aliases may have their own runtime.
CI actions are pinned to upstream commit SHAs, verified from their upstream tags.

The separately declared Rust minimum is **1.91**. Build, test and warnings-denied
Clippy passed locally with 1.91.0; CI has an explicit locked-graph MSRV job.
The locked Usage 6.11.1 crates declare 1.91, so a successful experimental 1.89
build with `--ignore-rust-version` is not a supported minimum. An experimental
1.88 build fails on the standard-library file-lock API (stable since 1.89).
Inside the checkout, rustup still selects 1.98.1 unless explicitly overridden.

| Archive | Build/test environment | Runtime support claim |
| --- | --- | --- |
| aarch64-apple-darwin | macos-15 ARM runner | macOS 15.0 or newer (binary minimum); ARM64 |
| x86_64-apple-darwin | macos-15-intel runner | macOS 15.0 or newer (binary minimum); Intel |
| x86_64-unknown-linux-gnu | ubuntu-22.04 x86-64 runner | Ubuntu 22.04/glibc 2.35 or newer compatible x86-64 environment |

Both macOS builds set MACOSX_DEPLOYMENT_TARGET=15.0, and the package smoke test
requires the binary's own `LC_BUILD_VERSION` minimum to be 15.0 (the Homebrew
formula also requires Sequoia). That minimum is not a claim of testing every
15.x release. The local ARM archive passed on macOS 15.7.4 ARM64; the Intel archive passed
under Rosetta, not on native Intel hardware. Native Intel and Linux claims become verified only
when their release matrix package tests pass; they are publication gates, not
inferred from a successful ARM build. Linux packages must run on the 22.04 build
runner; do not advertise older glibc compatibility. Windows, Linux ARM and older
macOS releases are deferred.

Every archive records source commit, the packaged binary's SHA-256, compiler,
Cargo, Usage, SDK/libc, actual runner image, the macOS minimum read from the
binary and lock hashes in BUILD-INFO.json. Hosted runner
images and SDK patch levels can change: this is a pinned toolchain plus recorded
inputs, not a bit-for-bit reproducible-build claim. Re-run source/package tests
and review the recorded inputs before changing tool versions or minimums.

## Hosted runner lifetimes

Hosted image retirement dates change. Check the authoritative
[runner-images announcements](https://github.com/actions/runner-images/issues?q=is%3Aissue+label%3AAnnouncement)
before release. If Ubuntu 22.04 runners become unavailable, use a reviewed
Ubuntu 22.04 container or equivalent glibc 2.35 toolchain to retain the baseline;
revisit native Intel support if Intel macOS runners become unavailable. A new
runner image is a build-input change requiring the same native package gates.

## Signing

macOS artifacts are unsigned/unnotarized in the Developer ID sense (a linker may
supply ad-hoc signing). Downloads may face Gatekeeper restrictions. See the
installation and signing policy; no security bypass is part of installation.
