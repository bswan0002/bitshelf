# Let scratch material expire

**Use this when** some material is only useful for a while (scratch notes, pasted prompts, [handoffs](handoffs.md)) and you'd like it cleaned up on a schedule instead of by hand.

## Set up

```sh
bs shelf add tmp --retention 14d
```

Every new bit in `tmp` gets an explicit `expires` timestamp 14 days after creation. Nothing is deleted yet.

## Clean up

```sh
bs prune tmp --dry-run --json    # see what would be removed
bs prune tmp                     # remove expired bits
```

Pruning removes files permanently; they don't go to the trash. Always preview first, and keep backups if you might want something back. To keep a bit, move it to a permanent shelf before it expires. Moves preserve `expires`, but permanent shelves are never pruned.

## Schedule it

bitshelf never runs in the background. To prune automatically, use your system scheduler. Examples for cron and launchd are in [Expiration and pruning](../concepts/pruning.md#scheduling).

## Limits

- Retention accepts whole days only (`14d`).
- Changing retention affects only bits created afterward.
- Bits without a valid `expires` value are reported and skipped, never guessed from file modification times.

See [Expiration and pruning](../concepts/pruning.md) for complete rules.
