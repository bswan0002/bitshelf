# Archive without deleting

**Use this when** material is finished but might matter again, and you want it out of everyday results without deleting it.

## The finished experience

```sh
bs archive notes/checklist
```

The bit moves to `archive/notes.checklist` and records where it came from. Default `bs list` and `bs search` no longer show it, but it's one flag away when you need it. With the agent skill installed, “archive the release checklist in bs” does the same thing.

## What each part does

| Part | Supplied by | Role |
| --- | --- | --- |
| `archive` shelf | You | An ordinary shelf |
| `discoverable = false` | You, in `archive/bs.toml` | Leaves the shelf out of default list/search results |
| `archive` alias | You, in global config | Shortcut for a `bs move` with a naming pattern and a `moved_from` field |
| Move validation and safety | `bs` | Refuses collisions and preserves the body and history |

Archive isn't a special bit state. It's an ordinary shelf, a discovery setting, and an alias.

## Set up

1. Create the shelf:

   ```sh
   bs shelf add archive
   ```

2. In `archive/bs.toml`, set at the top level (outside any `[tag_rules.*]` table):

   ```toml
   discoverable = false
   ```

   Leave `retention` unset for a permanent archive.

3. Add an alias to your global configuration (`~/.config/bitshelf/config.toml`):

   ```toml
   [aliases]
   archive = ["move", "{id}", "archive/{shelf}.{name}", "--set", "moved_from={id}"]
   ```

4. Preview, then archive:

   ```sh
   bs archive notes/checklist --dry-run --json
   bs archive notes/checklist
   ```

## Find archived material

```sh
bs list archive                  # explicit shelf selection always includes it
bs search checklist --all        # include non-discoverable shelves
bs search checklist --shelf archive
bs show archive/notes.checklist  # direct access always works
```

`discoverable = false` also hides the shelf from `bs open --pick`. It's not an access restriction: shelf listing, completion, context, show/open by ID, editing, sync, validation, and pruning all still include it.

## Restore

Look at `moved_from`, then move the bit back explicitly:

```sh
bs show archive/notes.checklist
bs move archive/notes.checklist notes/checklist --dry-run
bs move archive/notes.checklist notes/checklist
```

Restoring follows the same rules as any move: the destination shelf's requirements apply, and an occupied ID is refused. `moved_from` stays as ordinary metadata.

## Limits

- The `shelf.name` naming pattern and the `moved_from` field belong to this recipe, not to `bs`.
- Dots in existing names can cause destination collisions. Moves refuse them rather than choosing another name.
- Archiving an already archived bit is just another move and replaces `moved_from`.
- Moving changes the ID, and references to the old ID aren't rewritten.

See [Moving and aliases](../concepts/moving.md) for complete move and alias behavior.
