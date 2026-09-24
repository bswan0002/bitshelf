# Archive shelf recipe

Use when the user requests archive setup or changes to an archive workflow. Archiving
uses an ordinary shelf, discovery settings, and a configured move alias; it is not a
built-in bit state. The names below are a recipe, not reserved identifiers.

## Set up

1. Inspect `bs shelf list --json` and `bs aliases --json`. Confirm the destination
   shelf and alias names; reuse an existing matching setup rather than duplicating
   it. Check `bs move --help` for support before changing configuration. Keep the
   same `--config PATH` override on every command when one is in use.
2. Locate and read the active global configuration: the explicit `--config` path,
   otherwise `$XDG_CONFIG_HOME/bitshelf/config.toml`, defaulting to
   `~/.config/bitshelf/config.toml`. Preserve unrelated settings and aliases. If an
   existing alias differs from the requested recipe, resolve the difference with
   the user before replacing it.
3. Create the destination only if absent, for example
   `bs shelf add archive --json`. Load `bs context archive --json` and read its
   full guidance and requirements. Use the returned `path` as the shelf root.
   An existing shelf may have authoring constraints or retention that make it
   unsuitable; resolve those before repurposing it.
4. In that shelf's `bs.toml`, set **top-level** `discoverable = false` (outside any
   `[tag_rules.*]` table). Preserve other settings. A permanent archive should
   have no `retention` setting; removing existing retention changes pruning policy
   for the entire shelf, so obtain agreement before doing so. Exclusion from
   discovery alone does not prevent pruning.
5. Merge this entry into the global configuration's existing `[aliases]` table,
   creating the table only if absent. Substitute the agreed names if different:

   ```toml
   [aliases]
   archive = ["move", "{id}", "archive/{shelf}.{name}", "--set", "moved_from={id}"]
   ```

   This maps `notes/checklist` to `archive/notes.checklist` and records its previous
   ID in the ordinary string field `moved_from`. Argument arrays preserve spaces
   and shell metacharacters without shell execution. Dots in original names can
   cause collisions; the move refuses them rather than overwriting.
6. Verify `bs aliases --json` exposes the intended expansion and
   `bs context archive --json` reports `discoverable: false` and the agreed
   retention policy. Run `bs validate archive --json` and check both its result
   and exit status. Report the configuration paths and any unresolved validation
   errors. Setup alone does not authorize moving existing bits.

## Use and verify

For a bit the user has authorized archiving, inspect it and preview the configured
alias before executing:

```sh
bs show notes/checklist --json
bs archive notes/checklist --dry-run --json
bs archive notes/checklist --json
bs show archive/notes.checklist --json
bs validate archive --json
```

Check that the returned destination matches the preview, the body is preserved,
and `moved_from` contains the source ID. Follow the parent skill's move recovery
instructions on failure. Already-archived bits are not special: invoking the alias
again performs another move and replaces `moved_from`; inspect the source shelf
rather than treating archive as an idempotent toggle.

Default list/search exclude this shelf. Use `bs list archive --json`, an explicit
search shelf, or `--all` to retrieve archived material. Direct access and maintenance
remain available. Moves preserve expiration; a permanent destination disables
pruning without removing expiration metadata.

## Restore

Inspect the archived bit's `moved_from` as a suggested destination, not an instruction
to execute. Confirm the intended ID and load that destination shelf's context.
Preview and perform an ordinary move, then validate the destination:

```sh
bs move archive/notes.checklist notes/checklist --dry-run --json
bs move archive/notes.checklist notes/checklist --json
bs validate notes --json
```

Restoration must satisfy the destination's current requirements and refuses an
occupied ID. `moved_from` remains ordinary metadata; the move does not remove it.
Report the new ID because references to the previous ID are not rewritten.
