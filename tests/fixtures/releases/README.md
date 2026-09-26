# Release contract corpus

0.1.0 is the prepared first-release fixture, not evidence that a release has
already shipped. Freeze this directory at the approved release commit. Future
patch releases run every published fixture unchanged; new releases add cases or
version directories without rewriting historical expected bytes. If a minor
release intentionally breaks a contract, document the exact boundary and retain
read-only preservation tests for earlier plain Markdown.

The fixture includes a relative store config, explicit built-in alias, shelf
rules/guidance, an excluded shelf, missing dates, offset/future dates, Unicode and
CRLF/no-final-newline body bytes, and unknown nested metadata with full-range
integers. expected.body is the byte-level oracle. unsupported-config.toml proves
unknown format/config fields fail clearly rather than being silently ignored.
No prototype history/migrations are part of the corpus.
