# Get started

This walks through the core loop: keep something, find it from another context, and use or update it. It takes about five minutes. Use a disposable store while bitshelf is a prototype.

## 1. Install and set up

Install from source (see [Install](install.md) for other routes and shell completion):

```sh
cargo install --path . --locked      # from a bitshelf checkout
bs init --store ~/bitshelf
```

`bs init` writes `~/.config/bitshelf/config.toml`, creates the store directory, and adds a `notes` shelf. Run it without arguments for interactive setup. It never overwrites an existing configuration.

A store is a directory of **shelves**; each shelf holds **bits**, which are Markdown files:

```text
~/bitshelf/
└── notes/
    ├── bs.toml          # shelf settings
    └── bits/
        └── pagination-research.md
```

## 2. Keep something

Say you're in the middle of a session and have findings worth keeping:

```sh
bs add notes/pagination-research --title 'Cursor pagination research' --tags api --file findings.md
```

The ID `notes/pagination-research` is the bit's name—you choose it. `--file` takes the Markdown **body**; `bs` adds frontmatter with your title and tags plus automatic `created`/`updated` timestamps. Use `--file -` to read from stdin, or `bs add --interactive` to write in your editor.

## 3. Find it from another context

Later—another session, another worktree, another repository—it's still there:

```sh
bs list --long                  # every bit, with titles
bs list notes --tag api         # filter by shelf and tag
bs search 'cursor api'          # terms can match title and tags
bs show notes/pagination-research --body
```

::: tip Find it without remembering the wording
`bs search 'cursor api'` finds the bit above even though `cursor` is in its title and `api` is a tag. Results are relevance-ranked. Narrow with `--tag api` or `!draft`, use `bs search '"cursor pagination"'` for a phrase, or broaden with `--any`. See [Finding and reading](concepts/finding.md#search).
:::

## 4. Use it and build on it

The payoff is using the material again instead of recreating it. When it changes, update it in place rather than saving a copy:

```sh
bs edit notes/pagination-research                 # open in your editor
bs edit notes/pagination-research --file revised.md  # or replace the body
```

Edits preserve the title, tags, and other metadata, and advance `updated` only when something really changed. You can also edit the file directly in any editor—run `bs sync` afterward so timestamps stay accurate. See [Timestamps and sync](concepts/timestamps.md).

## 5. Give a shelf its own conventions

Shelves become more useful when they describe how their contents should be kept. Create one with a purpose and requirements:

```sh
bs shelf add snippets --description 'Reusable code from past work' --required title,tags
```

Then add a `SHELF.md` beside its `bits/` directory, written for whoever—person or agent—adds to it:

```markdown
# Snippets

Save code with enough context to reuse it elsewhere: dependencies,
assumptions, and an example call site. Tag each snippet with its
language and framework. Update an existing snippet rather than
saving a near-duplicate.
```

`bs context snippets` shows the shelf's description, requirements, and full guidance. Requirements such as `--required title,tags` are enforced by `bs`; guidance is followed by the people and agents who read it.

## 6. Bring in an agent

Install the agent skill, and the same loop works through requests:

> “Save this retry helper to bs snippets.”
>
> “Look in bs for a retry helper we can use here.”

The agent loads the shelf's guidance before writing, searches before creating duplicates, and validates what it saved. See [Using with agents](agents.md).

## Next

- Browse the [recipes](recipes/design-docs.md) for complete workflows.
- Read [Shelves, bits, and configuration](concepts/shelves.md) for how the store is organized.
