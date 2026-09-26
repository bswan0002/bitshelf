# Pre-release verification

The safety regression suite uses temporary stores only. `cargo test --locked` covers strict YAML corpus/JSON, exact bodies, command-local dates/import/restore, ID round trips, incomplete shelf discovery, destination tag replacement, prune conflicts and partial outcomes, symlinks, editor conflicts/recovery, real terminals, shell completion and helper aliases. `tests/locking.rs` tests two competing processes and release after SIGINT/SIGKILL.

Fault injection is compiled only into unit tests. `filesystem::tests` exercises temp creation, writes, permission application, file sync, publication, removal and post-publication directory-sync failure. `moving::tests` distinguishes safe rollback from source-unlink durability failure. `completion::commit_tests` preserves shell settings on failed writes/stale previews. `prune::operation_tests` interposes body/expiry changes and deletion failure after an earlier successful removal.

Run `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, `cargo build --locked`, `bash scripts/generate-docs.sh`, then check `git diff --exit-code -- docs/reference` after committing intended generated changes, and `npm ci && npm run docs:build`. CI runs source tests on Linux, macOS ARM and macOS Intel. Local execution is evidence only for the host platform; unexecuted CI jobs are not claimed as passed.

Residual boundaries: trusted local filesystems, best-effort checks against external editors, no hostile-path compare-and-swap, no distributed locking, no multi-file transaction, and bounded directory durability as documented in safety. Package/installation gates are separate from source tests.

## Actual package checks

`package.py` produces archives only from a clean checkout by default;
`--allow-dirty` explicitly labels local development previews in BUILD-INFO.json.
`smoke-package.py` extracts each archive into an isolated temporary home with a
minimal PATH and verifies commands, architecture, file modes, man page, shell
completions, docs, skill version, dependency and Rust runtime notices. The release
workflow runs this against all three real artifacts and gates publication on
actual formula installation, `brew test`, revision upgrade, retest and uninstall
on each native runner. Prereleases run archive checks and skip the stable formula.

Local preparation on macOS 15.7.4 ARM64 passed both macOS archive smoke tests
(the Intel binary ran under Rosetta). The formula generated from those real
checksums passed Apple Silicon install/test/revision-upgrade/test/uninstall.
The temporary tap was removed. Linux and native Intel execution remain mandatory
release-workflow gates; neither has been run locally. No synthetic Linux archive
or checksum was used as installation evidence. The local two-target checksum set
is intentionally insufficient for production publication.

## Candidate revision checks (ticket 28)

The revised 0.1.0 candidate passed locally on macOS 15.7.4 ARM64:

- Formatting, warnings-denied Clippy, locked build and all 98 Rust tests on the
  pinned 1.98.1 toolchain; locked build/tests/Clippy also passed on MSRV 1.91.0.
- Generated-reference drift check, clean npm install and documentation build;
  13 release/formula/tap script tests, workflow lint and dependency-notice checks.
- Both newly built macOS archives passed extraction/smoke checks; Intel execution
  was under Rosetta. The real local checksums drove a fresh Apple Silicon
  Homebrew install/test/controlled revision-upgrade/test/uninstall lifecycle.
  The temporary tap was removed; this is not a published-tap or fresh-machine test.
- Before replacing the sole prototype install with explicit maintainer approval,
  the candidate read and validated all 44 existing bits, including excluded
  shelves, and checked shelf settings/tag rules, aliases, the ticket helper and
  an archive-alias dry run. `validate` without a shelf includes every shelf;
  unlike `list`, it has no `--all` flag. No store incompatibility was found.
  The matching skill replaced the prototype copy; seven shelf-helper tests passed
  after removing bare-array input support.
- Unicode regressions probe filesystem normalization rather than infer it from
  the OS. This local filesystem exercised collision refusal; byte-distinct
  filesystem behavior remains covered by the conditional regression and the
  native Linux CI gate, not claimed as locally executed evidence.

The incomplete-discovery audit found no scope bug or reason to change semantics.
`src/store.rs` excludes hidden entries, auxiliary shelf files, non-`.md` entries
and nested content; recognized but unsafe/unsupported bit paths remain diagnostics.
Existing regressions `auxiliary_files_are_ignored_and_preserved`,
`discovered_ids_roundtrip_and_unsupported_entries_are_never_ids`,
`non_utf8_filenames_are_diagnostics_in_all_output_modes`, and
`partial_discovery_keeps_healthy_shelves_and_explicit_failures_are_clear`, together
with symlink tests, passed. Incomplete scans still return healthy results with
exit 1 and `complete: false`.

Native Intel/Linux source and package runs, the Linux MSRV job, and a quarantined
browser download have not been executed locally. Signing remains deferred;
`spctl --assess` rejected the ad-hoc-signed candidate during preparation. No push,
tag, workflow dispatch, release publication or website deployment was performed.

## Strict CLI parsing follow-up (ticket 29)

The flag-typo issue was reproduced before the fix in an isolated temporary store:
`shelf add --bogus` and `add --bogus/x --stdin` both created content and exited 0.
After the fix, the regression matrix verifies unknown long/short/attached options
exit 2 across built-ins and built-in aliases, with a recursive filesystem snapshot
unchanged after each failure. Explicit `--` operands and `--shelf=-notes` work;
ID-alias substitution preserves positional and option-value roles, including
flag-looking names. External helper argv remains literal on both sides of `--`.
Validation help now explains its all-shelves scope; validation/prune retain their
per-item arrays, including failure outcomes.

Local macOS 15.7.4 ARM64 verification passed again: formatting, locked build,
warnings-denied Clippy and all **103 Rust tests** on both 1.98.1 and 1.91.0;
generated-reference drift check, npm install/docs build, 13 release/formula/tap
script tests, workflow lint and notices checks. Newly built ARM and Intel archives
passed isolated extraction/smoke checks (Intel under Rosetta). Their actual local
checksums passed Homebrew install/test/revision-upgrade/test/uninstall; the temporary
tap was removed. The installed executable and skill were refreshed together after
read-only real-store validation/listing, ticket-helper and archive-alias dry-run
checks; all seven shelf-helper tests passed.

Native Intel/Linux and fresh-machine published-tap checks remain release gates.
No push, tag, workflow dispatch or deployment occurred. Handoff retention was
configured separately by the maintainer and was not modified by this work.


## Prerelease audit blocker fixes

Local macOS ARM64 verification of the working-tree fixes based on `8258db2`:

- All **105 Rust tests** pass on 1.98.1 and MSRV 1.91.0. Regressions cover
  attached values on value-less flags (including shell-install consent and
  built-in alias templates), unchanged filesystem snapshots on rejection,
  non-canonical integer rejection without rewriting the source, and BOM-prefixed
  LF/CRLF frontmatter edits preserving metadata and exact body bytes.
- Formatting, locked build and warnings-denied Clippy pass. Generated command
  references have no drift. Clean npm install/documentation build, local Markdown
  link checks, all 13 release/formula/tap script tests, workflow lint, and
  regenerated dependency-notice checks pass.
- Candidate banners now advise backups rather than disposable-only prototype
  use. The README points to candidate docs instead of the stale live site.
  **The live site has not been redeployed and remains a release gate.**

The boolean-flag fix is a documented local patch to usage-argv 6.11.1; dependency
notices and inventory have been regenerated. Binary, underscore-separated and
leading-zero decimal integers must be rewritten in supported notation or quoted
as strings before mutation. Quoted spellings are preserved.

This pass does not claim new archive/Homebrew, native Intel/Linux, remote workflow
or publication verification. Nonblocking audit findings remain outside this pass.
No installed executable, real store, remote configuration, tag or deployment was
changed.
