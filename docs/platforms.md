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
| aarch64-apple-darwin | macos-15 ARM runner | macOS 15.7.4 or newer; ARM64 |
| x86_64-apple-darwin | macos-15-intel runner | macOS 15 runner version recorded in BUILD-INFO.json or newer; Intel |
| x86_64-unknown-linux-gnu | ubuntu-22.04 x86-64 runner | Ubuntu 22.04/glibc 2.35 or newer compatible x86-64 environment |

Both macOS builds set MACOSX_DEPLOYMENT_TARGET=15.0. That binary load-command
minimum is not a claim of testing every 15.x release. The local ARM archive passed on macOS 15.7.4 ARM64; the Intel archive passed
under Rosetta, not on native Intel hardware. Native Intel and Linux claims become verified only
when their release matrix package tests pass; they are publication gates, not
inferred from a successful ARM build. Linux packages must run on the 22.04 build
runner; do not advertise older glibc compatibility. Windows, Linux ARM and older
macOS releases are deferred.

Every archive records source commit, compiler, Cargo, Usage, SDK/libc, actual
runner image, deployment target and lock hashes in BUILD-INFO.json. Hosted runner
images and SDK patch levels can change: this is a pinned toolchain plus recorded
inputs, not a bit-for-bit reproducible-build claim. Re-run source/package tests
and review the recorded inputs before changing tool versions or minimums.

macOS artifacts are unsigned/unnotarized in the Developer ID sense (a linker may
supply ad-hoc signing). Downloads may face Gatekeeper restrictions. See the
installation and signing policy; no security bypass is part of installation.
