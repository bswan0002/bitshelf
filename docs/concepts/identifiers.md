# Identifier contract

An ID is exactly `shelf/name`, without the final `.md` extension. Both components must be nonempty UTF-8, not start with a dot, not end with `.md`, and contain no slash, backslash or Unicode control character. Spaces, embedded dots, leading hyphens and Unicode are accepted; quote shell arguments and use `--` for leading-hyphen arguments. Hidden entries are intentionally ignored (including recovery drafts). Bit files have exactly a lowercase `.md` extension; nested directories are not bits.

Unknown options are usage errors (exit 2), not names. Put every option before
`--` when a positional operand starts with a hyphen:

```sh
bs shelf add -- -notes
bs add --title 'Example' --file note.md -- -notes/bit
bs show --body -- -notes/bit
bs move --dry-run -- -notes/bit notes/bit
```

An ID such as `notes/-bit` does not itself start with a hyphen and needs no
separator. For a flag's hyphen-prefixed value use the attached form, for example
`bs search bit --shelf=-notes` or `--title=-example`. Built-in aliases follow the
same operand convention; explicit helper aliases preserve the helper's argv.

Discovered IDs pass the same checks as explicit arguments and resolve to their original files. Explicit IDs and shelf names must match the stored name exactly: on case-insensitive or normalizing filesystems (such as default APFS), a variant spelling like `notes/FOO` for a stored `notes/Foo` is refused with an error naming the stored ID rather than resolving to it, and `bs add` refuses a variant of an existing name. Names that are empty or only whitespace are rejected. Unsupported names, `.md` directories/special files and symlinks are diagnostics, never lossy fabricated IDs. Validation reports the real path and a null ID when no valid ID exists. JSON paths for unrepresentable OS strings are diagnostic display strings, not round-trippable addresses. `--paths --null` emits exact OS path bytes for supported discovered entries; it is not a raw filesystem enumerator for unsupported filenames. Use filesystem tools to repair unsupported names.

Names retain case and Unicode representation; bs does not normalize either. Filesystem case sensitivity and Unicode normalization apply. No-clobber checks reject collisions, including case-only moves on case-insensitive filesystems. Case-only moves on case-sensitive filesystems are ordinary moves. Use an intermediate distinct name where needed. No promise is made that every accepted name is portable between filesystems or operating systems; avoid spaces, punctuation and case/normalization-only distinctions when portability matters.

Normalization regressions probe the filesystem: composed/decomposed names round-trip byte-exactly when distinct; saves and moves refuse collisions when equivalent. Tests also cover spaces, dots, leading hyphens, Unicode, controls/newlines, backslashes, hidden names, extension rules, non-UTF-8 names, symlinks and `.md` directories. Completion, validation, list/search, show/edit/move/prune share this policy.
