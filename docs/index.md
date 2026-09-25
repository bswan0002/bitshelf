# bitshelf

> **Prototype — do not rely on bitshelf yet.** Commands, configuration, and file formats may change without backward compatibility, and bugs may cause data loss. Use disposable data. This site tracks `main` and may describe unreleased work.

Agents let us pursue more work in parallel: more sessions, more worktrees, more repositories. Useful work now outlives the conversation that produced it, and it's easy to lose.

- A context window is filling up, and there's an insight worth keeping for a session you haven't started.
- You built a nice component before realizing the task no longer needs it.
- Research done in a throwaway worktree needs to reach a session in a different one.
- Design documents should inform implementation across sessions and repositories.

**bitshelf gives that work a home outside any one session.** You or your agent deliberately put something on a shelf; later, from any context, either of you finds it, reads it, and builds on it. The executable is `bs`.

## Three ideas

**Information outlives the session.** Keep a note, a handoff, a design doc, or a piece of code independently of the conversation, repository, or worktree it came from.

**People and agents use the same material.** A shelf is ordinary Markdown in a directory you own. You browse and edit it in your editor; agents use the same `bs` commands with JSON output. Neither needs the other to read it.

**Each shelf has its own working conventions.** A shelf can describe, in plain language, how its contents should be written, updated, and found—and can enforce structure such as required tags. A design-doc shelf, a handoff shelf, and a snippet shelf each work the way their contents need.

## Deliberate, not automatic memory

bitshelf is not an agent memory system. Nothing is saved unless you, or an agent you asked, saves it. Nothing is loaded into a conversation unless it's looked up. Agents discover what shelves exist, read a shelf's guidance when it's relevant, search, and retrieve only the bits they need—having a store doesn't mean loading it everywhere.

You stay in control of what's kept, and what's kept stays inspectable.

## The loop

```sh
# Session A, in one worktree: keep something worth keeping.
bs add notes/pagination-research --tags api --file findings.md

# Days later, session B, in another repository: find it and use it.
bs search pagination
bs show notes/pagination-research --body

# Build on it rather than duplicating it.
bs edit notes/pagination-research --file revised.md
```

With the agent skill installed, the same loop is a request: *“save these pagination findings to bs”* in one session, *“check bs for our pagination research”* in the next. The second use is the payoff. [Get started →](start.md)

## Why not just a folder of Markdown?

It *is* a folder of Markdown—browse it, grep it, back it up, or commit it to Git, with or without `bs`. No account, database, or background service. What `bs` adds is what makes that folder dependable for both people and agents:

- **Conventions agents actually load.** `bs context SHELF` hands an agent a shelf's full guidance and requirements before it writes.
- **Structure that's checked.** Shelves can require titles or tags and restrict tag values, for example to known repository names. Invalid saves are rejected.
- **Stable identifiers and JSON.** Every bit is `shelf/name`, and every command has structured output for agents and scripts.
- **Exact content.** Prompts and code are stored byte-for-byte; metadata lives in frontmatter.
- **Discovery you control.** An archive shelf can stay out of default results while remaining accessible.

## Where next

- [Get started](start.md): set up a store and complete the loop.
- [Recipes](recipes/design-docs.md): design docs, handoffs, and reusable code, end to end.
- [Using with agents](agents.md): install the skill and see how agents use shelves.
- [Reference](concepts/shelves.md): complete behavior, limits, and recovery.
