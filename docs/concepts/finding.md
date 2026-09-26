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
bs search 'release checklist' --shelf notes --long
bs search 'release checklist !draft' --tag workflow
bs search '"release checklist"'       # contiguous phrase, not two separate terms
bs search 'pagination cursor' --any   # either term
bs search checklist --all --json
bs search checklist --sort id        # identifier order for scripts
```

Search is case-insensitive. By default, **every whitespace-separated term** must appear somewhere in the bit's ID, optional title, tags, or body; terms can match different fields and appear in any order. `release checklist` finds `release-checklist` and prose with the words apart. Terms are substrings, not whole words; there is no stemming, regex, or fuzzy character matching.

- **Phrases:** double quotes preserve a contiguous substring, including punctuation and whitespace. `"release checklist"` does not match `release-checklist`. A phrase must fit within one field or one tag.
- **Exclusions:** prefix a term or phrase with `!`, such as `!draft` or `!"old version"`. Any matching exclusion rejects the bit, even with `--any`. An exclusion-only query returns all remaining bits.
- **Broader matching:** `--any` requires at least one positive term rather than all.
- **Separators:** unquoted terms also match IDs and tags with `-`, `_`, `.`, `/`, and `:` treated as spaces. For example, `project_bitshelf` matches the tag `project:bitshelf`. Quoted phrases bypass this normalization.
- **Scope:** `--shelf` selects a shelf; `--tag` requires an exact, case-sensitive tag, like `bs list --tag`. These filters apply regardless of `--any`.

Pass the query as one shell argument. Single shell quotes around it preserve search's double quotes and `!`. Within the query, backslash escapes the next character: `bs search '\!important'` searches for a literal `!important`, and `bs search '"say \"hello\""'` searches for `say "hello"`. Use `\\` for a literal backslash. Empty queries/terms, unfinished escapes, unmatched quotes, and quotes not separated into their own terms are usage errors (exit 2).

### Relevance and match details

Results default to descending relevance, with identifier as the tie-breaker. Each distinct positive term contributes its strongest field's weight: **ID/title 8, tags 4, body 1**. Scores are summed across terms; repeated occurrences and duplicate terms do not inflate ranking. Use `--sort id` for identifier order.

JSON results include `matches.fields` (which fields matched positive terms) and `matches.score`; bodies are omitted. Use these signals to choose candidates, then `bs show` to read them. Exclusion-only results have empty match fields and score 0. See the [JSON contract](../json.md).

A search with no matches succeeds with empty text output or JSON `[]`. There's no search index, dependency, or daemon; every search reads the current files.

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
