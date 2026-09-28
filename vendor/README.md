# Local dependency patches

`demand/` is Demand 2.1.0 from crates.io, under its included MIT license.

The patch in `src/input.rs` removes the redundant `reset_cursor_to_end()` from successful Enter handling. `handle_submit()` calls `clear()`, which already performs that movement. Moving twice leaves the original prompt title behind when the completed prompt is rendered.

Input also holds the existing Unix raw-mode guard for the whole prompt, starting before the first frame, instead of relying only on Console's per-key raw mode. Fast input could otherwise echo between frames and leave a duplicate title. The guard restores the original terminal settings on return or error; `event.rs` exposes it within the crate for this use.

The root Cargo.toml applies this source via `[patch.crates-io]`; no installed registry files are modified. `tests/terminal.rs` exercises real PTYs and an ANSI terminal screen to catch duplicated prompt titles. Remove the vendor patch when an upstream version includes the fix.

`usage-argv/` is usage-argv 6.11.1 from crates.io (MIT; license text in
`licenses/upstream/usage-6.11.1-LICENSE`, upstream notice included). The patch
in `src/lib.rs::long_flag` rejects attached values on value-less flags,
including implicit help/version and negated switches, instead of discarding
them. It uses the existing unknown-flag diagnostic for the invalid spelling.
This fixes both normal CLI parsing and built-in alias template binding without
maintaining a second CLI grammar. `tests/cli.rs` covers rejection before any
store or shell configuration changes, and literal option values remain valid.
Remove this patch when upstream rejects these spellings.
