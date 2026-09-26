# bitshelf

> **Early development — do not use bitshelf yet.** Commands, configuration, and file formats may change without backward compatibility, and bugs may cause data loss. Use disposable data only.

Agents let us work on more things in parallel, and useful work now outlives the session that produced it. **bitshelf gives that work a home outside any one conversation, repository, or worktree**: a store of ordinary Markdown shelves that people and agents both use through the `bs` executable.

- **Information outlives the session.** Keep notes, design docs, reusable code, exact prompts, and handoffs where any later session can find them.
- **People and agents use the same material.** It's a directory you own: browse it in your editor, or let agents use `bs` with JSON output.
- **Each shelf has its own working conventions.** `SHELF.md` guidance tells people and agents how to keep a shelf's contents; `bs.toml` enforces structure such as required tags.

Saving and retrieving are deliberate. This isn't automatic agent memory.

**Documentation:** [bitshelf.benswanson.dev](https://bitshelf.benswanson.dev) · [Get started](docs/start.md) · [Recipes](docs/recipes/design-docs.md) · [Using with agents](docs/agents.md)

## Quick start (development only)

Install a current stable [Rust toolchain](https://rustup.rs), then:

```sh
git clone https://github.com/bswan0002/bitshelf.git
cd bitshelf
cargo install --path . --locked
bs completion install        # optional; start a new shell afterward
bs init --store ~/bitshelf
```

Keep something, then find it from another context:

```sh
bs add notes/pagination-research --tags api --file findings.md
bs search 'pagination api'   # terms can match across fields; relevance-ranked
bs show notes/pagination-research --body
bs edit notes/pagination-research --file revised.md
```

Install the agent skill separately:

```sh
npx skills add bswan0002/bitshelf --skill bitshelf --global
```

## Documentation

- [Why bitshelf](docs/index.md) and [Get started](docs/start.md)
- Recipes: [design docs](docs/recipes/design-docs.md), [handoffs](docs/recipes/handoffs.md), [reusable code](docs/recipes/reusable-code.md), [archive](docs/recipes/archive.md), [expiring shelves](docs/recipes/expiring-shelves.md)
- Reference: [shelves and configuration](docs/concepts/shelves.md), [editing](docs/concepts/editing.md), [finding](docs/concepts/finding.md), [timestamps](docs/concepts/timestamps.md), [moving and aliases](docs/concepts/moving.md), [pruning](docs/concepts/pruning.md), [completion](docs/concepts/completion.md), [safety and recovery](docs/concepts/safety.md), [JSON contract](docs/json.md)
- [Command reference](docs/reference/index.md) (generated from Usage declarations)
- [Install](docs/install.md) · [Releasing](RELEASING.md)

## Development

Use Node.js 22.12+ for the documentation commands. `docs:dev` starts a long-running server; stop it before running the next command.

```sh
cargo test --locked
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo build
cargo install usage-cli --version 6.11.1 --locked
bash scripts/generate-docs.sh
npm ci
npm run docs:dev
npm run docs:build
```

The Rust CLI is synchronous, one application package, with no search index; hidden store-local state supports timestamp change detection. A small documented Demand rendering patch lives in `vendor/demand` until fixed upstream. Tests use temporary stores, not your notes. Documentation pins VitePress 2 alpha. GitHub Actions is configured to test Linux/macOS, build release archives on version tags, and deploy development docs separately. Nothing is published or scheduled by local setup.

Initial targets are macOS Apple Silicon, macOS Intel, and Linux x86-64. See [Safety and recovery](docs/concepts/safety.md) for prototype boundaries.
