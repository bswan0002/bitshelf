# Local dependency patches

`demand/` is Demand 2.1.0 from crates.io, under its included MIT license.

One patch in `src/input.rs` removes the redundant `reset_cursor_to_end()` from successful Enter handling. `handle_submit()` calls `clear()`, which already performs that movement. Moving twice leaves the original prompt title behind when the completed prompt is rendered.

The root Cargo.toml applies this source via `[patch.crates-io]`; no installed registry files are modified. `tests/terminal.rs` exercises real PTYs and an ANSI terminal screen to catch duplicated prompt titles. Remove the vendor patch when an upstream version includes the fix.
