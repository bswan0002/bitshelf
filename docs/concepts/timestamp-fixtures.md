# Timestamp implementation fixtures

Specification for tickets 08/09 and regression ticket 17; these cases are not yet
executable coverage. Use injected clocks rather than sleeps. For every failure,
assert source/destination bytes and draft recovery behavior, not just exit status.
See the normative [timestamp contract](timestamps.md).

Use original `created = 2026-01-01T00:00:00Z`,
`updated = 2026-02-01T00:00:00+00:00`, and command finalization time
`2026-03-01T00:00:00.123456Z` unless overridden.

| Fixture | Expected result |
| --- | --- |
| Add body, empty body, CRLF, missing final newline | Both dates equal finalization time; body bytes preserved |
| Interactive add opened before finalization; several draft saves | Both dates use finalization clock, not open/intermediate-save clock |
| Add draft has historical, invalid, or removed date fields | Generated dates replace draft date fields; malformed YAML still fails |
| Edit body or user metadata | Preserve created; updated equals finalization time |
| Edit changes expires or an unknown nested field | Counts as user-metadata change; stamp updated; no automatic retention extension |
| Exact no-op, including body-only input equal to original | Preserve entire source bytes; no rewrite |
| YAML comment, mapping-order, nested-order, or quoting-only edit | Preserve dates; explicit formatting edits may be published |
| Sequence reorder, scalar type change, field absence vs. null | Counts as a change where supported by ticket 02 |
| Body newline, whitespace, CRLF/LF, or final-newline change | Counts as a change; preserve proposed body bytes |
| External edit before command; command makes no further change | No timestamp repair or historical comparison |
| Plain move across IDs/shelves, including differing retention defaults | Preserve dates and absence; no stamp from identity change |
| Move with actual user-metadata change vs. same-value assignment | Stamp only the actual change |
| Imported file with neither date, created only, or updated only | No-op/plain move preserves absence; changed content sets updated only |
| Valid imported non-UTC offset and fractional precision | Untouched dates retain string values; generated dates use UTC |
| Missing date required by shelf | Reject final candidate unless supplied under applicable command rules |
| Invalid date: null, number, empty, date-only, malformed offset | Edit/move fails, including no-op; automatic stamp cannot mask invalid original |
| Draft explicitly repairs/removes invalid original date | Accept when resulting metadata satisfies shelf requirements |
| Draft explicitly edits/removes created only | Honor change; no stamp unless body/user metadata also changes |
| Draft explicitly edits/removes updated only | Honor change; no automatic stamp |
| Draft changes body and explicitly changes/removes updated | Explicit updated wins, even if older than original or command time |
| Draft rewrites date string to equivalent instant | Honor different string representation; quoting alone is not an override |
| Draft leaves updated unchanged while changing created and body | Honor created; automatically stamp updated |
| Draft temporarily changes dates/content then restores originals | Final draft is a no-op; intermediate changes irrelevant |
| Clock earlier than or equal to original updated | Changed content saves; updated retains original value/representation |
| Prior updated has subsecond precision beyond generated default | Clamp never rounds backward |
| Missing updated with future created | Set updated to command time; no cross-field chronology restriction |
| Editor nonzero exit or invalid final draft | Source unchanged; retain recovery draft |
| Concurrent source change while editor is open | Reject finalization; preserve external source and recoverable draft |
| Add/move collision or pre-publication write failure | Existing files and their dates unchanged |
| Publication succeeds, durability confirmation or cleanup fails | Report committed/uncertain outcome accurately; no promised rollback |
| Open, list, show, search, validate | No timestamp or history writes |
| No state, stale state, corrupt state, obsolete state.lock | Same timestamp behavior; prototype files untouched and never consulted |
| Delete/recreate at same identifier | Add gets new dates; no inherited history |
| Removed sync command | Ordinary unknown-command failure; no reconciliation or migration |

Run command-boundary cases against both human and JSON output where supported.
Exercise editor finalization using a deterministic editor helper. Lock release,
durable publication, move partial outcomes, and platform failure injection are
owned by tickets 10–13/17; they must not reintroduce lifecycle bookkeeping.
