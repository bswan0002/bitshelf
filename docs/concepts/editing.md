# Saving and editing


## Saving exact content

```sh
bs add notes/release-checklist --tags release --file checklist.md
printf '%s' 'An exact prompt' | bs add tmp/original-prompt --file -
bs add notes/draft --interactive
```

- `add` requires a body source: `--file <FILE>` reads a **body**; `--file -` and `--stdin` read stdin; `--interactive` opens a draft. `--file` and `--stdin` are mutually exclusive. Without a source, `add` fails rather than silently saving an empty body; use `--file /dev/null` for an intentionally empty bit.
- The body is preserved byte-for-byte for valid UTF-8 Markdown, including CRLF line endings and a missing final newline.
- Input is a body, not a frontmatter document to merge. `bs` writes the frontmatter: optional title and tags, `created` and `updated` set to the creation time (UTC), and `expires` on retention shelves.
- ID collisions fail; choose a different ID.
- `--title <TITLE>` adds optional descriptive metadata. It's required only when the shelf requires `title`, and a supplied title must be nonempty. The ID remains the display name, and changing a title never renames a bit.
- `--tags a,b` sets tags, checked against the shelf's [tag rules](shelves.md#namespaced-tag-rules).
- `--set <KEY=VALUE>` (literal string), `--set-json <KEY=JSON>` (typed value) and `--unset <KEY>` change other frontmatter fields on `add`, `edit` and `move`. See [Metadata mutations](metadata.md).
- Invalid metadata is rejected before saving.

## Editing

```sh
bs edit notes/release-checklist                          # draft in your editor; waits
bs edit notes/release-checklist --file revised.md        # replace body, keep metadata
bs edit notes/release-checklist --stdin --json < revised.md
bs edit notes/release-checklist --title 'New title' --tags release,checklist --json
```

- `--file` (including `--file -`) and `--stdin` take **body-only** input, like `add`, and are mutually exclusive.
- `--title` and `--tags` replace those fields. An empty tags argument clears tags. `--set`, `--set-json` and `--unset` work as on `add`.
- If `created` or `updated` holds an invalid value, edits fail until it's repaired: `bs edit <ID> --unset updated` (or `created`) removes it without an editor. Valid dates stay reserved.
- Unrelated metadata and expiration are preserved; editing never extends retention.
- Body/user-metadata changes automatically stamp `updated`, clamped to the original value if the clock moves backward. No-op edits preserve dates and missing dates. Explicit valid timestamp changes/removals in an editor draft are honored, including an explicit `updated` override. See [Timestamps](timestamps.md).
- With no mutation flags, `bs edit <ID>` opens a temporary draft in your editor and waits. The original is replaced only after the editor exits successfully, the metadata validates, and a check confirms the original didn't change concurrently. A failed edit keeps the draft and reports its path.
- JSON mode requires explicit mutation flags and never launches an editor.

Editing files directly in any editor is also supported. Markdown is authoritative: manage dates yourself for external edits. There is no watcher or sync.

## Editors

Editor precedence is config `editor`, then `VISUAL`, then `EDITOR`.

```toml
editor = "code --reuse-window"          # string: quoted-argument parsing
editor = ["code", "--reuse-window"]     # array: each argument literal
```

- Strings use quoted-argument parsing, not shell evaluation. Quote executable paths that contain spaces. Command substitutions and expansions are never executed. Environment editor strings are parsed the same way.
- Saving configuration normalizes either form to an array.
- Paths are appended as separate arguments.
- Interactive `bs init` skips the editor question when `VISUAL` or `EDITOR` is set to a nonblank value, without copying that value into config. Otherwise you can choose an editor or leave it blank.

### Opening files

```sh
bs open                           # store directory
bs open ui                        # shelf directory
bs open ui/menu notes/checklist   # multiple files
bs open --pick                    # filtered multiselection
```

The editor must support opening directories and multiple files for those forms. Opening a shelf doesn't expand it into every file. `bs open` never adds waiting flags and never manages timestamps; dates for subsequent external edits are the user's responsibility.

### Waiting for GUI editors

For interactive add and editor-mode `bs edit`, `bs` must wait until you finish editing. It automatically adds `--wait` to `code`, `code-insiders`, `codium`, `cursor`, and `subl`, including absolute paths to them. Existing `--wait` or `-w` arguments aren't duplicated. Common terminal editors (`vi`, `vim`, `nvim`, `nano`, `pico`, `hx`, `helix`, `micro`) are unchanged.

This is a list of known commands, not general GUI detection. Unknown editors and wrappers are left unchanged, and `bs` warns that they must keep running until editing finishes; configure their blocking flag yourself. Shell wrappers aren't inspected or rewritten. A GUI command that returns immediately can't safely finalize a draft.

## Interactive add

`bs add --interactive` prompts for a shelf, bit name, and tags, then opens a draft. `bs add notes/draft --interactive` skips shelf and name selection. It doesn't ask for a title; pass `--title` or add one in the draft if the shelf requires it. Cancelling during the prompts creates nothing.

JSON mode and piped or noninteractive invocations never prompt.

Both creation timestamps are generated together at finalization, not when the
editor opens. Interactive add overwrites draft `created`/`updated` values or
removals; editor-mode edit instead honors explicit valid date edits. Intermediate
draft saves never update source dates.

## Drafts

Drafts are temporary recovery files, not a notes feature. Interactive add opens a hidden `.draft-*.md`, and editor-mode edit opens a hidden `.edit-*.md` copy of the original. Both live in the shelf's `bits/` directory and are excluded from discovery and pruning. If saving fails, the draft's path is printed on stderr. See [Safety and recovery](safety.md#recovering-a-draft) before retrying. A failure to remove a draft after a successful save is only a warning.
