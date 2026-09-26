# Pre-release verification

The safety regression suite uses temporary stores only. `cargo test --locked` covers strict YAML corpus/JSON, exact bodies, command-local dates/import/restore, ID round trips, incomplete shelf discovery, destination tag replacement, prune conflicts and partial outcomes, symlinks, editor conflicts/recovery, real terminals, shell completion and helper aliases. `tests/locking.rs` tests two competing processes and release after SIGINT/SIGKILL.

Fault injection is compiled only into unit tests. `filesystem::tests` exercises temp creation, writes, permission application, file sync, publication, removal and post-publication directory-sync failure. `moving::tests` distinguishes safe rollback from source-unlink durability failure. `completion::commit_tests` preserves shell settings on failed writes/stale previews. `prune::operation_tests` interposes body/expiry changes and deletion failure after an earlier successful removal.

Run `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, `cargo build --locked`, `bash scripts/generate-docs.sh`, then check `git diff --exit-code -- docs/reference` after committing intended generated changes, and `npm ci && npm run docs:build`. CI runs source tests on Linux, macOS ARM and macOS Intel. Local execution is evidence only for the host platform; unexecuted CI jobs are not claimed as passed.

Residual boundaries: trusted local filesystems, best-effort checks against external editors, no hostile-path compare-and-swap, no distributed locking, no multi-file transaction, and bounded directory durability as documented in safety. Package/installation gates are separate from source tests.
