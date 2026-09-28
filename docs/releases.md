# Match documentation and skill to your executable

Run `bs --version`. The archive for that exact version contains `docs/` and
`skills/bitshelf/`; these snapshots record the matching contract and agent guidance.
Homebrew bundles the same files under `$(brew --prefix bitshelf)/share/bitshelf`.
Bundling the skill files does not configure an agent to use them; install the
skill for your agents with the `npx` command below.
`BUILD-INFO.json` records the exact source commit and package build inputs.

The corresponding Git tag is `v` followed by the executable version. For example,
the release documentation for 0.1.0 is at
[the v0.1.0 tag](https://github.com/bswan0002/bitshelf/tree/v0.1.0/docs).
Repository main and the website are explicitly development documentation and may
describe features absent from an installed release.

Use the `npx` installer with the matching tag. For `bs 0.1.0`:

```sh
npx skills add https://github.com/bswan0002/bitshelf/tree/v0.1.0 --skill bitshelf --global
```

Choose your agents when prompted. Replace `v0.1.0` with the tag matching
`bs --version` when installing another release. The skill's `VERSION` file
identifies the bundled CLI version. Installing a skill does not install `bs`.
Do not automatically update a skill from main while keeping an older binary.
For source development, run `npx skills add . --skill bitshelf --global` from
the same checkout as the binary. Read the matching changelog before a minor upgrade.

Nothing in the local build publishes docs or a skill. The docs workflow builds on
main but deploys only through manual dispatch with `deploy=true` and the Pages
environment. Release archives carry their docs without depending on the website.
