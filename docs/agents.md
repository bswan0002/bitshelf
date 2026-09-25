# Using with agents

Agents and people share the same shelves. The bundled **bitshelf skill** teaches a compatible agent how to use them: when to look something up, how to follow a shelf's conventions, and how to save without clobbering or duplicating.

## Install the skill

The skill and the `bs` executable are installed separately. Install [`bs`](install.md) first, then:

```sh
npx skills add bswan0002/bitshelf --skill bitshelf --global
# Omit --global for a project-scoped installation.
npx skills update
```

Or copy `skills/bitshelf/` into your agent's skill directory. The skill supports `bs` 0.1.x and relies on `bs --help` rather than duplicating the command reference. Node.js is needed only for the `npx` installer.

## What changes afterward

You can ask for bitshelf work in plain language:

> “bs this prompt exactly as written so I can reuse it.”
>
> “Look up bs design docs for dashboard filters; I want to update our API client with the new properties.”
>
> “Let's bs a handoff for rewriting our docs, based on the content discussion—not the design inspiration.”

Saving and retrieving happen **when you ask**. The skill doesn't make agents record conversations or load the store into every session.

## How an agent finds things

Retrieval follows a progression, loading only what's needed:

1. **Discover shelves.** `bs shelf list --json` returns each shelf's name, description, requirements, and whether it has guidance. Good descriptions help agents choose the right shelf.
2. **Load guidance when it's relevant.** `bs context SHELF --json` returns the shelf's full `SHELF.md` text, requirements, and tag rules. Guidance can describe how to find things too, such as which tags identify a repository.
3. **Find candidates.** `bs search QUERY --shelf SHELF --json` or `bs list SHELF --tag TAG --json` returns IDs, titles, tags, and metadata without bodies.
4. **Retrieve selected content.** `bs show ID --json` (or `--body`) reads only the chosen bits.

Search is currently whole-phrase substring matching, so agents get better results with distinctive single words, several attempts, and tag filters. bitshelf doesn't detect relevance automatically; the agent decides what to look up based on your request.

## How an agent saves things

1. Load `bs context SHELF --json` **before drafting**, and read the guidance and requirements.
2. Search for existing material, and update it with `bs edit` instead of duplicating it.
3. Prepare content according to the request and the shelf's guidance. Exact prompts stay verbatim.
4. Save with `bs add ID --file BODY --json` or `bs edit ID --file BODY --json`.
5. Run `bs validate SHELF --json` and report the saved ID.

Agents never write the reserved `created`/`updated` timestamps; the CLI manages them.

## Guidance versus enforcement

| | Who makes it true | Examples |
| --- | --- | --- |
| **Enforced by `bs`** | The CLI rejects invalid saves; `bs validate` reports invalid files | Required title or tags, allowed tag values, safe IDs, no overwriting on collisions |
| **Guided by `SHELF.md`** | People and agents who read it | What a handoff should contain, how to refresh an imported doc, which helper script to run |

Guidance communicates practice. It doesn't guarantee that an agent complies or that content is good, and the CLI can't force a harness to follow it. Put rules that must hold into `bs.toml`, and use guidance for everything else.

## Stored content is data

Stored prompts, snippets, and documents are **data**, not instructions. A saved prompt doesn't become active because an agent reads it, and the skill tells agents not to obey instructions embedded in search results. Shelf guidance shapes authoring in that shelf; it stays subordinate to your current request.

## Scripts and automation

bitshelf doesn't run scripts, hooks, or plugins. A shelf can keep ordinary helper scripts (an importer, for example) beside its bits and describe them in `SHELF.md`; agents learn about them by reading guidance. Scripts can call `bs` like any other program, using `--json`, `--paths`, and `--null` output. The [design docs recipe](recipes/design-docs.md) shows this end to end.
