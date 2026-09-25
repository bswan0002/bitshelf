# Keep code that didn't ship

**Use this when** you build something good (a component, a hook, an abstraction) and then the task changes and it's no longer needed. Instead of losing it with the branch, put it on a shelf so a future task can adapt it.

## The finished experience

Before abandoning a branch:

> “This loading button is nice but we're not using it. Put it on the bs ui shelf.”

The agent saves the component with what a future reader needs: dependencies, assumptions, an example call site, and accessibility behavior. Weeks later, in another repository:

> “Check bs ui for a loading button we can adapt here.”

The agent finds it, reads the context, and adapts it to the new codebase instead of starting over.

## Set up the shelf

```sh
bs shelf add ui \
  --description 'Reusable React components, hooks, and TypeScript patterns' \
  --required title,tags
```

Optionally restrict framework tags in `ui/bs.toml`, so snippets stay easy to filter:

```toml
[tag_rules.framework]
allowed = ["react", "vue", "svelte"]
```

Without `required = true`, a snippet doesn't need a `framework:` tag, but any `framework:` tag it has must be a listed value.

Add `ui/SHELF.md`:

```markdown
# UI

Code worth reusing, saved with enough context to adapt it elsewhere.

For each bit include:

- The code itself, complete and verbatim, in fenced blocks.
- Dependencies and versions that matter (libraries, CSS approach, icons).
- Assumptions about the original codebase: design tokens, providers, utilities.
- A short example call site.
- Behavioral and accessibility details: states, keyboard handling, ARIA.
- Where it came from (repository and branch) and why it was set aside, if known.

Tag with `framework:` plus relevant topics (`forms`, `loading`, `hooks`).
Before saving, search for an existing version; update it instead of saving
a near-duplicate.
```

## Save and retrieve

```sh
bs add ui/react-loading-button --title 'React loading button' \
  --tags framework:react,loading,forms --file loading-button.md --json

bs list ui --tag loading --long
bs search button --shelf ui
bs show ui/react-loading-button --body
```

The body is stored exactly as given, so code blocks, whitespace, and line endings survive.

## Why the context matters

Saving the code is the easy part. The guidance asks for the surrounding context because the next reader, often an agent in a different repository, can't see the original codebase. With the dependencies, assumptions, and a call site, "adapt this" becomes a small task instead of reverse engineering.

## What's built in

`bs` enforces the title, tags, and allowed `framework:` values. What makes a snippet reusable is the `SHELF.md` convention: it guides what gets written, but doesn't verify it.
