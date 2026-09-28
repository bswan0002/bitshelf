# Expiration and pruning

For the workflow, see [Let scratch material expire](../recipes/expiring-shelves.md).

## Retention

```sh
bs shelf add tmp --retention 14d
```

- Retention accepts positive whole days only (`14d`), up to `36500d`.
- New bits in the shelf get an explicit `expires` timestamp at creation plus the retention period.
- Changing retention affects only future bits. Removing it (and any `on_expire` setting) from `bs.toml` disables pruning for that shelf.
- Edits and moves preserve `expires` and never extend it.

## Pruning

```sh
bs prune tmp --dry-run --json
bs prune tmp --json
bs prune --dry-run            # all shelves, same rules
```

- Only bits with a valid explicit `expires` at or before the current time are eligible.
- On retention shelves, missing or invalid expiration, expired bits with otherwise invalid metadata, and unreadable bit entries are reported as `skipped` with an error (one row per bit), and prune exits 1. Fix a missing date with `bs edit <ID> --set expires=<RFC 3339>`.
- Permanent shelves are never prune candidates: their bits, valid or not, produce no rows and don't make prune fail. Use `bs validate` to find invalid bits there.
- Unreadable shelves or shelf configuration are reported as `skipped` rows with `id: null` because their retention is unknown.
- Guidance, hidden drafts, symlinks, and bits that haven't expired yet are left alone.
- Modification times are never used as a guess.
- By default, pruning **deletes** files; nothing goes to the trash. Configure a move action below to archive instead. Back up your store if you need recovery.

## Archive on expiry

Create a permanent destination, then edit the source shelf's `bs.toml`:

```sh
bs shelf add archive
```

```toml
retention = "14d"
on_expire = { move = "archive/{shelf}.{name}" }
```

Set `on_expire` at the top level, outside any `[tag_rules.*]` table. The only supported actions are `"delete"` (also the default when omitted) and `{ move = "DESTINATION" }`. A move destination must render to a full `SHELF/NAME` ID. Only `{shelf}` and `{name}` placeholders are supported, expanded once as literal strings. No aliases, shell commands, hooks, or metadata options are accepted. `on_expire` requires `retention`.

- Run `bs prune <SHELF> --dry-run --json` after configuring the policy. Previews resolve destinations and check collisions and destination requirements. The destination shelf must already exist; `bs validate` checks policy syntax, while prune's preview checks destinations for eligible bits.
- An expiry move preserves body, metadata, timestamps, permissions, and `expires`, just like a plain `bs move`. It does not add `moved_from`. Use a **permanent archive shelf** to keep moved bits indefinitely: a retention-enabled destination may act on them on the next prune.
- Destinations are never overwritten. Collisions (including collisions caused by dots in names), missing shelves, and destination validation failures leave the source intact, report `skipped`, and exit 1. They continue failing until resolved. Failed archival never falls back to deletion.
- Prune processes a snapshot: newly moved bits are not discovered again during the same invocation. Changing `on_expire` changes what happens to existing expired bits on the next run; it does not rewrite their expiration dates.
- `discoverable = false` can hide the archive from default list/search, but does not disable pruning. See [the archive recipe](../recipes/archive.md).

## Outcomes and recovery

Prune captures candidates' original bytes, then checks them before acting and rechecks shelf settings. A changed body, expiration, or policy is skipped. The writer lock covers the batch; built-in moves reuse it rather than invoking a second process. Dry runs acquire no lock and are not reservations.

JSON remains a per-item array. Move rows include `destination` (the resolved destination ID); `id` and `path` always identify the source. Statuses are:

| Status | Meaning |
| --- | --- |
| `would_remove`, `would_move` | Validated dry-run operation |
| `removed`, `moved` | Completed operation |
| `skipped` | Failed validation, changed source/settings, or an operation that did not complete safely; inspect `error` |
| `removed_with_error` | Unlinked, but source-directory durability is uncertain |
| `moved_with_error` | Destination saved and source removed, but source-directory durability is uncertain |
| `move_uncertain` | Destination publication or rollback failed; inspect both paths before retrying |

Failures include `error` and exit 1. Ordinary skips allow other candidates to proceed. Uncertain publication, rollback, or durability **stops the batch**; later candidates remain untouched and have no result rows. Earlier outcomes remain in the array. An error does not mean nothing changed, and no batch rollback is implied.

## Scheduling

`bs` never installs a scheduler or runs in the background. Use your system's scheduler, and run a dry run manually first.

**cron** (replace the paths; a scheduler's `PATH` and `HOME` may differ from your shell's):

```cron
0 9 * * * /home/you/.cargo/bin/bs --config /home/you/.config/bitshelf/config.toml prune --json >> /home/you/bitshelf-prune.log 2>&1
```

**macOS launchd.** Save as `~/Library/LaunchAgents/local.bitshelf.prune.plist`, replacing every `/Users/you`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>local.bitshelf.prune</string>
  <key>ProgramArguments</key><array>
    <string>/Users/you/.cargo/bin/bs</string>
    <string>--config</string><string>/Users/you/.config/bitshelf/config.toml</string>
    <string>prune</string><string>--json</string>
  </array>
  <key>StartCalendarInterval</key><dict><key>Hour</key><integer>9</integer><key>Minute</key><integer>0</integer></dict>
  <key>StandardOutPath</key><string>/Users/you/Library/Logs/bitshelf-prune.log</string>
  <key>StandardErrorPath</key><string>/Users/you/Library/Logs/bitshelf-prune-errors.log</string>
</dict></plist>
```

```sh
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/local.bitshelf.prune.plist   # load
launchctl bootout gui/$(id -u) ~/Library/LaunchAgents/local.bitshelf.prune.plist     # remove
```
