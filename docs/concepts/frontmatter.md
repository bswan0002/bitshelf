# Frontmatter contract

Frontmatter starts only with `---` and LF or CRLF at byte zero and ends at a line containing exactly `---` (LF, CRLF or EOF). An opener without a closer is invalid. Plain Markdown otherwise has empty metadata; a leading Markdown horizontal rule therefore needs a matching closer or must be preceded by a blank line.

Supported values are null, booleans, UTF-8 strings, finite IEEE-754 binary64 floats, integers in signed 64-bit or unsigned 64-bit range, arrays, and recursively string-keyed mappings. Integers outside that range are rejected, rather than rounded into floats. Unquoted binary (`0b101`), underscore-separated (`1_000`), and leading-zero decimal (`012`) integer spellings are rejected. Use decimal without leading zeros or separators, supported `0x`/`0o` notation, or quote the value to preserve its spelling as a string. Unknown field names are preserved. Quoting a scalar forces string interpretation. YAML comments, quoting, key order and layout may normalize on mutation; the UTF-8 body bytes, including CRLF and lack of final newline, are preserved.

Duplicate mapping keys, non-string keys, explicit YAML tags, merge keys (`<<`), and non-finite floats are rejected at every depth. Anchors and aliases are accepted as expanded values, with no identity semantics; cyclic aliases are invalid. Alias expansion is subject to the parser's resource limits. YAML 1.2 scalar rules apply (e.g. `yes` is a string). Decimal/exponent floats carry binary64 precision, not arbitrary precision.

Every accepted value must survive JSON inspection, rendering/reparse, edit, move and semantic comparison without type/value loss. Invalid metadata has diagnostics, never a success-shaped null substitution; raw `show` remains available for repair. Semantic comparison ignores mapping order, comments and the two command-owned timestamp fields. It includes all other supported metadata and exact body bytes.

The fixture corpus in `tests/fixtures/frontmatter` is normative. The parser, renderer and mutation regressions execute this corpus.

A leading UTF-8 BOM before the opening frontmatter delimiter is recognized and removed when metadata is rewritten. BOMs in plain bodies are preserved.

Implementation uses serde-saphyr 1.3.0 (MIT OR Apache-2.0) and its granit-parser 1.3.0 scanner. Both are pinned in Cargo.lock; the upstream project has current releases and resource-budgeted alias expansion. See [upstream documentation](https://docs.rs/serde-saphyr/1.3.0/serde_saphyr/). The scanner rejects explicit tags and integer overflow; a recursive Serde visitor rejects non-string keys, duplicates, merges and non-finite values. Serialization may quote timestamp strings; their string values and body bytes remain unchanged. Run `cargo test metadata::tests::frontmatter_corpus` when updating either dependency.
