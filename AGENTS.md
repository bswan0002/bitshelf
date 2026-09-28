# bitshelf

`bs` is a local, Markdown-first CLI for storing notes, snippets, prompts, and handoffs. The implementation and tests are the specification.

**There are no users yet. Backward compatibility is not a requirement. Prefer a clean design over compatibility layers or migration machinery.**

## CLI usage notation

Use uppercase placeholders consistently in help, docs, and alias usage strings, including paths and input types (`CONFIG`, `PATH`, `TOML`). Use the same bracket conventions as generated help: `<SHELF>` for a required positional argument, `[SHELF]` for an optional one, and `[ARGS]...` for optional repeated arguments. Requiredness follows the documented invocation: `bs add <ID> --file <PATH>` requires an ID even though general help shows `[ID]` because `--interactive` can prompt for it. Option values use angle brackets (`--sort <SORT>`); bracket the whole option when showing it as optional (`[--sort <SORT>]`). Preserve actual accepted syntax, such as `[--count=<COUNT>]` when `=` is required. Preserve useful format information when normalizing notation: keep `expires=<RFC 3339>` rather than replacing it with `expires=<TIMESTAMP>`, and retain assignment syntax such as `--set <KEY=VALUE>`. Describe constrained formats, units, separators, and allowed values in the placeholder or nearby prose, based on the implementation and tests. Commands, option names, and concrete example values retain their literal spelling; template variables such as `{{ args }}` are not usage placeholders.
