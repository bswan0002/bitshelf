# Match documentation and skill to your executable

Run `bs --version`. The archive for that exact version contains `docs/` and
`skills/bitshelf/`; use those copies for the matching contract and agent guidance.
Homebrew installs the same snapshots under `$(brew --prefix bitshelf)/share/bitshelf`.
`BUILD-INFO.json` records the exact source commit and package build inputs.

The corresponding Git tag is `v` followed by the executable version. For example,
the release documentation for 0.1.0 is at
[the v0.1.0 tag](https://github.com/bswan0002/bitshelf/tree/v0.1.0/docs).
Repository main and the website are explicitly development documentation and may
describe features absent from an installed release.

Copy the entire matching `skills/bitshelf/` directory into your agent's personal
or project skill directory. Its `VERSION` file identifies the bundled CLI version.
A skill alone does not install bs. Do not automatically update a skill from main
while keeping an older binary. For source development, use the skill from the
same checkout as the binary. Read the matching changelog before a minor upgrade.

Nothing in the local build publishes docs or a skill. The docs workflow builds on
main but deploys only through manual dispatch with `deploy=true` and the Pages
environment. Release archives carry their docs without depending on the website.
