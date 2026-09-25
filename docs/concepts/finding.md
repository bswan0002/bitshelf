# Finding and reading

## Listing

```sh
bs list                              # every discoverable bit, one ID per line
bs list notes --long                 # ID<TAB>title
bs list notes --tag release          # exact tag match
bs list --sort updated --reverse     # most recently edited first
bs list --sort created --reverse     # newest first
bs list --all                        # include non-discoverable shelves
```

List defaults to identifier order. Timestamp sorting is ascending unless `--reverse` is given, breaks ties by identifier, and places missing or invalid dates last in either direction. Sorting doesn't sync; run [`bs sync`](timestamps.md) first if files were edited directly.

## Search

```sh
bs search checklist
bs search 'release checklist' --shelf notes --long
bs search checklist --all --json
```

Search is case-insensitive, plain-text matching of the **whole query as one substring** against each bit's ID, optional title, tags, and body. Results are sorted by identifier. A search with no matches succeeds with empty output.

Because the query must appear verbatim, `release checklist` won't match a bit that only contains `release-checklist` or the two words apart. Prefer one distinctive word, try variants, and combine search with `--shelf`, `list --tag`, and `list --long`. Better multi-term matching is planned.

There's no search index; every search reads the current files.

## Discovery

Default `list`, `search`, and `open --pick` omit shelves with `discoverable = false`. Pass `--all` or select the shelf explicitly (`bs list archive`, `--shelf archive`) to include them. See [the archive recipe](../recipes/archive.md).

## Reading

```sh
bs show notes/release-checklist           # the entire file, unmodified
bs show notes/release-checklist --body    # body only, exactly as stored
bs show notes/release-checklist --json    # {id, path, content}
```

`--body` removes frontmatter and preserves the body exactly, including line endings and a missing final newline. A file without frontmatter is all body. Malformed frontmatter makes `--body` fail rather than guess. With `--json`, the same selection is returned in `content`.

## Output for pipelines

`list` and `search` print one ID per line by default.

| Flag | Output |
| --- | --- |
| `--long` | `ID<TAB>title`; a missing title leaves the column empty |
| `--paths` | Absolute file paths instead of IDs |
| `--null` | NUL-terminates each ID or path, including the last, for tools such as `xargs -0` |
| `--json` | Structured results; see the [JSON contract](../json.md) |

`--json` can't be combined with the text flags, and `--long` can't be combined with `--paths` or `--null`. Empty results produce no output. Use `--null` for safe composition, and JSON rather than parsing titles.

```sh
bs list notes --paths --null | xargs -0 grep -l 'pattern'
bs show notes/release-checklist --body | bs add notes/checklist-copy --file -
printf '%s' 'Replacement body' | bs edit notes/checklist-copy --file -
```

If the consumer closes the pipe early, `bs` exits quietly with status 0; other output errors are still failures. Results are collected in memory; these flags don't make processing incremental.

Read-only commands (`list`, `search`, `show`, `validate`) never change files or timestamps.
