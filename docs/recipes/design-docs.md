# Design docs ready for implementation

**Use this when** design documents live somewhere else (Confluence, Google Docs, a wiki) and you want them on hand, in Markdown, whenever an agent implements against them, across sessions and repositories.

## The finished experience

In any repository, you ask:

> “Look up bs design docs for dashboard filters; I want to update our API client with the new properties.”

The agent lists the `design-docs` shelf filtered by initiative and repository, reads the matching document, and implements against it. When the source page changes, you ask it to refresh the document, and it updates the existing bit instead of creating a second copy.

The value isn't saving a web page. It's keeping reference material consistent and easy to find while you implement.

## What each part does

| Part | Supplied by | Role |
| --- | --- | --- |
| `design-docs/bs.toml` | You | Requires titles and tags; restricts `initiative:` and `repo:` tags to known values |
| `design-docs/SHELF.md` | You | Tells agents how to import, attribute, tag, and refresh documents |
| `scripts/export-confluence-to-md.py` | You | Fetches a page and prints clean Markdown. An ordinary script, not a bitshelf integration |
| Importing, checking, saving | The agent | Runs the helper, inspects the output, applies conventions, saves through `bs` |
| Validation | `bs` | Rejects missing titles, missing required tags, or unknown tag values |

`bs` never discovers or runs the script. The agent learns about it by reading `SHELF.md`.

## Set up the shelf

```sh
bs shelf add design-docs \
  --description 'Product and technical design docs, for use during implementation' \
  --required title,tags
```

Add tag rules to `design-docs/bs.toml`:

```toml
[tag_rules.initiative]
required = true
allowed = ["dashboard-filters", "billing-v2"]

[tag_rules.repo]
required = true
allowed = ["api-client", "web-app"]
```

Each document must now have at least one `initiative:` tag and one `repo:` tag, and only listed values are accepted. A document can apply to several repositories (`repo:api-client,repo:web-app`). Edit this file when you add an initiative or repository. See [namespaced tag rules](../concepts/shelves.md#namespaced-tag-rules).

Add a helper and describe it in `design-docs/SHELF.md`:

```text
design-docs/
├── bs.toml
├── SHELF.md
├── bits/
└── scripts/
    └── export-confluence-to-md.py
```

```markdown
# Design docs

Design documents imported from Confluence, used as implementation references.

## Importing or refreshing a page

1. Search this shelf for the page's source URL first. If a bit already has it,
   refresh that bit with `bs edit` instead of adding a new one.
2. Run `python3 scripts/export-confluence-to-md.py --help` from this shelf's
   root for prerequisites. It reads ATLASSIAN_EMAIL and ATLASSIAN_API_KEY from
   the environment and prints a Markdown body to stdout. It does not save.
3. Check the output: headings, tables, code blocks, and links should survive,
   with no login pages or navigation.
4. Start the body with `Source: <page URL>` and `Last imported: <date>`.
5. Save with a title matching the page title, one `initiative:` tag, and a
   `repo:` tag for each repository the document affects.

## Using documents

When implementing, prefer the document's stated contracts over guesses, and
point out anything in the code that contradicts the document.
```

## Import a document

This is what the agent runs, following the guidance (you can run it yourself too):

```sh
body=$(mktemp)
python3 scripts/export-confluence-to-md.py 'https://example.atlassian.net/wiki/…' > "$body"
# inspect "$body" before saving
bs add design-docs/dashboard-filters --title 'Dashboard filters' \
  --tags initiative:dashboard-filters,repo:api-client,repo:web-app --file "$body" --json
bs validate design-docs --json
```

If the tags don't satisfy the rules, `bs add` refuses the save and names the missing or invalid namespace.

## Use it later

```sh
bs list design-docs --tag repo:api-client --long
bs list design-docs --tag initiative:dashboard-filters --json
bs search filters --shelf design-docs
bs show design-docs/dashboard-filters --body
```

Tags are the most reliable way to narrow results, because [search](../concepts/finding.md#search) currently matches the whole query as one substring.

## Refresh in bulk

The same guidance supports refreshing many documents: ask the agent to refresh every document for an initiative, and it can list them by tag, re-run the helper for each `Source:` URL, and `bs edit` each one. `bs edit` keeps the existing tags and `created` date, and advances `updated` only when the content actually changed.

## Limits

- The helper is yours to write and maintain. bitshelf has no Confluence integration.
- A helper can also save through `bs` itself. Either way, say in `SHELF.md` whether it only prints content or also saves, so agents don't write the same document twice.
- Guidance describes the process; only `bs.toml` rules are enforced. Agents can still import poorly, so review important documents.
- Keep credentials in the environment, not in the shelf.
