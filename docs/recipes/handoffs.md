# Focused handoffs between sessions

**Use this when** a session has done useful thinking, but the work should continue somewhere fresh: the context window is filling, the next step belongs in another worktree, or you want a clean start without re-explaining everything.

## The finished experience

Near the end of a long conversation about documentation, you ask:

> “Let's bs a handoff for rewriting our docs, based on the latest message about content—not the design inspiration.”

The agent writes a focused brief to the `handoffs` shelf. It includes the task, the decisions made, the context needed, open questions, and next steps. It leaves out the unrelated tangents on purpose. In a fresh session:

> “Pick up the bs handoff for the docs rewrite.”

The new session starts with exactly what matters. Leaving things out is part of the value: a handoff is a selection, not a transcript summary.

## Set up the shelf

```sh
bs shelf add handoffs \
  --description 'Focused briefs for continuing work in a fresh session' \
  --required title \
  --retention 14d
```

Most handoffs are useful for days, not months, so the shelf starts with a retention period: every handoff gets an `expires` date when it's created. Leave out `--retention` if you want to keep handoffs until you delete or archive them.

Add `handoffs/SHELF.md`:

```markdown
# Handoffs

A handoff lets a fresh session continue specific work without the original
conversation. Write for a capable reader who has none of that context.

Include only what the next session needs:

- **Task:** what the next session should accomplish, in one or two sentences.
- **Decisions:** what was agreed, and why when the reason matters.
- **Context:** relevant files, links, commands, constraints, and any bitshelf
  IDs worth reading.
- **Open questions:** what is unresolved or needs the user's input.
- **Next steps:** a concrete starting point.

Leave out tangents, abandoned approaches (unless they prevent repeated
mistakes), and anything the user asked to exclude. Quote the user exactly
when their wording matters. Title the handoff after the task, not the session.
```

## What the artifact looks like

```markdown
# Docs rewrite: content direction

## Task
Restructure the documentation site around motivation, the core loop, and
recipes, keeping existing precision in a reference layer.

## Decisions
- Lead with continuity across sessions; do not narrow the pitch to handoffs.
- Answer "why not just a folder?" explicitly.
- Visual design is out of scope; see notes/bitshelf-aesthetic-direction.

## Context
- Direction: notes/bitshelf-docs-content-direction
- Current docs: docs/index.md, docs/guide.md, docs/releases.md

## Open questions
- Should the maintainer checklist leave the docs site?

## Next steps
Draft the new sidebar, then the homepage.
```

## Pick it up later

```sh
bs list handoffs --sort created --reverse --long   # newest first
bs show handoffs/docs-rewrite --body
```

Once the work is done, delete the bit, [archive it](archive.md), or let it expire (below).

## Let handoffs expire

Check what has expired, then prune:

```sh
bs prune handoffs --dry-run
bs prune handoffs
```

Nothing is deleted until you run `bs prune` yourself or schedule it. See [Let scratch material expire](expiring-shelves.md).

Retention sets `expires` only on bits created after it was configured. If you add `--retention` to an existing shelf (`bs shelf add handoffs --retention 14d`), older handoffs have no `expires`, and prune reports them as `skipped` and exits 1 on every run. Give each one an explicit RFC 3339 date with an offset, or delete or archive it:

```sh
bs edit handoffs/docs-rewrite --set expires=2026-10-09T00:00:00Z
```

## What's built in

The shelf, required title, IDs, and retention are enforced by `bs`. What a good handoff contains is a convention in `SHELF.md`. It shapes what agents write, but it can't guarantee quality, so skim a handoff before relying on it.
