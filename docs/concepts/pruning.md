# Expiration and pruning

For the workflow, see [Let scratch material expire](../recipes/expiring-shelves.md).

## Retention

```sh
bs shelf add tmp --retention 14d
```

- Retention accepts positive whole days only (`14d`).
- New bits in the shelf get an explicit `expires` timestamp at creation plus the retention period.
- Changing retention affects only future bits. Removing it (by editing `bs.toml`) disables pruning for that shelf.
- Edits, sync, and moves preserve `expires` and never extend it.

## Pruning

```sh
bs prune tmp --dry-run --json
bs prune tmp --json
bs prune --dry-run            # all shelves, same rules
```

- Only bits with a valid explicit `expires` at or before the current time are removed.
- Missing or invalid expiration, or otherwise invalid metadata, is reported as `skipped` with an error, and prune exits 1.
- Guidance, hidden drafts, permanent shelves, symlinks, and bits that haven't expired yet are left alone.
- Modification times are never used as a guess.
- Pruning **deletes** files; nothing goes to the trash. Back up your store if you need recovery.

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
