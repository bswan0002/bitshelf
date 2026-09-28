# Get started

This walks through the core loop: keep something, find it from another context, and use or update it. It takes about five minutes. This is the 0.1.0 release candidate. Back up your store before upgrading or running destructive commands; a temporary store is a good way to try this walkthrough.

## 1. Install

With [Rust installed](https://rustup.rs), install from source and set up shell completion:

```sh
cargo install --git https://github.com/bswan0002/bitshelf.git --locked && bs completion install
```

Completion setup previews changes and asks first. Start a new shell afterward. See [Install](install.md) for toolchain requirements and installing from a local checkout.

## 2. Install the agent skill globally (optional, strongly recommended)

The skill teaches your agent how to find and save material, follow shelf conventions, and avoid duplicates. Install it globally so it's available across repositories—not just in this project.

With Node.js and npm installed, run:

```sh
npx skills add bswan0002/bitshelf --skill bitshelf --global
```

Choose your agents when prompted. `--global` installs at the user level rather than in the current project. These commands install from the repository; update the executable and skill together so they stay matched. See [Using with agents](agents.md) and [release matching](releases.md).

## 3. Set up your store

```sh
bs init --store ~/bitshelf
```

`bs init` writes `$XDG_CONFIG_HOME/bitshelf/config.toml` (by default `~/.config/bitshelf/config.toml`), creates the store directory, and adds a `notes` shelf. Run it without arguments for interactive setup. It never overwrites an existing configuration.

A store is a directory of **shelves**; each shelf holds **bits**, which are Markdown files:

```text
~/bitshelf/
└── notes/
    ├── bs.toml          # shelf settings
    └── bits/
        └── pagination-research.md
```

## 4. Keep something

Say you're in the middle of a session and have findings worth keeping:

```sh
bs add notes/pagination-research --title 'Cursor pagination research' --tags api --file findings.md
```

The ID `notes/pagination-research` is the bit's name—you choose it. `--file` takes the Markdown **body**; `bs` adds frontmatter with your title and tags plus automatic `created`/`updated` timestamps. Use `--file -` to read from stdin, or `bs add --interactive` to write in your editor.

## 5. Find it from another context

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

## 6. Use it and build on it

The payoff is using the material again instead of recreating it. When it changes, update it in place rather than saving a copy:

```sh
bs edit notes/pagination-research                 # open in your editor
bs edit notes/pagination-research --file revised.md  # or replace the body
```

Edits preserve the title, tags, and other metadata, and advance `updated` only when something really changed. You can also edit the file directly in any editor—manage timestamps yourself for direct edits. See [Timestamps](concepts/timestamps.md).

## 7. Give a shelf its own conventions

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

## 8. Try it with an agent

With the agent skill installed, the same loop works through requests:

> “Save this retry helper to bs snippets.”
>
> “Look in bs for a retry helper we can use here.”

The agent loads the shelf's guidance before writing, searches before creating duplicates, and validates what it saved. See [Using with agents](agents.md).

## Next

- Browse the [recipes](recipes/design-docs.md) for complete workflows.
- Read [Shelves, bits, and configuration](concepts/shelves.md) for how the store is organized.
