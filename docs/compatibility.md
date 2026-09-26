# Release compatibility

There are no compatibility obligations to unreleased prototypes. The first public line is 0.1.x. Within a published minor line, patches preserve documented commands, configuration, supported frontmatter meaning, JSON fields/types, exit codes and helper argv/exit forwarding. Additive JSON fields and optional commands are permitted; consumers must ignore unknown fields. Ranking weights and prose diagnostic wording are implementation details. Search query semantics and deterministic tie ordering remain documented behavior.

A minor release may break commands/configuration/JSON, with explicit changelog and upgrade instructions. Unknown configuration fields are rejected rather than ignored. Unsupported metadata is diagnosed and never silently discarded. A future incompatible storage format must introduce a detectable version marker and fail closed in readers that do not understand it; no speculative format or migration engine is introduced now. Back up the entire store/configuration before crossing minor lines; downgrades require checking that line's supported metadata/configuration contract.

Partial collection output is a complete JSON document. List/search/shelf list use `{results, errors, complete}`; each scope error has `{path, error}`. `complete: false` means something was inaccessible or omitted and exits 1 after emitting healthy results. Invalid addressable bits remain in results with per-bit errors and also make the collection incomplete. Explicit bad shelf selection fails clearly. Validation and prune retain per-item arrays; any failure/skipped unsafe item exits 1. Prune reports successful removals even when later removals fail. Fatal setup errors emit stderr only. Codes for prose diagnostics are deferred: structural outcome fields plus exit status suffice for this release.

External helper aliases own their output and may not emit JSON. bs forwards literal argv, global config/executable context and exit status. Built-in aliases have their target's contract. Never parse human-readable diagnostics or depend on search weight constants.

## Upgrade and downgrade procedure

The checked-in `tests/fixtures/releases/0.1.0` corpus becomes the first published
baseline only at release approval; there are no earlier public compatibility
obligations. Preserve historical fixtures unchanged for subsequent patch tests.
Before a minor upgrade, back up the entire store and global/shelf configuration,
read its changelog, then run read-only validation against a copy. No migration runs
automatically. Unknown supported metadata survives authoring commands.

Downgrades within a published patch line follow the same contract; newly added
optional config keys must be removed only after consulting that version's docs.
Across minor lines, restore a compatible configuration/store backup or prove the
older reader accepts the actual files before writing. Unknown config fields fail
closed. A future incompatible storage format must require a new configuration
key/version or distinct representation that old readers reject; unknown ordinary
frontmatter fields alone are not a format boundary. No format marker is added now
because this release uses ordinary Markdown and has no incompatible public history.
