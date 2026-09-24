#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Keep this version aligned with usage-rs; older Usage CLIs cannot parse its spec.
# cargo install usage-cli --version 6.11.1 --locked
BS="${BS:-target/debug/bs}"
USAGE="${USAGE:-usage}"
mkdir -p dist/completions docs/reference
"$BS" __usage_spec__ > dist/bs.usage.kdl
"$USAGE" generate markdown --file dist/bs.usage.kdl --multi --out-dir docs/reference --url-prefix /reference
"$USAGE" generate manpage --file dist/bs.usage.kdl --out-file dist/bs.1
"$BS" completion --shell bash > dist/completions/bs.bash
"$BS" completion --shell zsh > dist/completions/_bs
"$BS" completion --shell fish > dist/completions/bs.fish
