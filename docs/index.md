# bitshelf

**Development documentation · prototype 0.1.x.** This site tracks main and may describe unreleased work.

A local Markdown-first store for reusable notes, snippets, prompts, and temporary handoffs. People and agents use the same files and the same `bs` executable.

```sh
cargo install --path . --locked && bs completion install
# Start a new shell to activate completion.
bs init --store ~/bitshelf --editor code
bs add notes/release-checklist --file checklist.md
bs search 'release' --json
bs open notes/release-checklist
```

Your store stays useful without bitshelf: browse, edit, search, back up, or commit ordinary Markdown files. No account, database, daemon, or automatic memory capture.

Start with the [user guide](guide.md), the [JSON contract](json.md), or the [generated CLI reference](reference/index.md). See [release status and installation](releases.md) before expecting a published binary or Homebrew formula.
