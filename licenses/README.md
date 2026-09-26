# Third-party notices

Run `python3.12 scripts/notices.py` after any dependency/lock change; review
`licenses/inventory.json` and `THIRD_PARTY_NOTICES.txt`, then commit both.
`python3.12 scripts/notices.py --check` fails on drift or missing source notices.
The generator uses Cargo's actual normal/build dependency tree for all three
supported release targets, excluding dev-only dependencies. Build dependencies
and nested source notices are included conservatively. Inventory entries record
package versions, license expressions, source repositories and hashes of every
included license/notice file. Cargo.lock itself is hashed too.

License texts are copied from the locked Cargo source packages. Usage 6.11.1's
published crates include third-party NOTICE files but omit their own root MIT
license; `upstream/usage-6.11.1-LICENSE` is the verbatim root license from the exact
commit in those packages' `.cargo_vcs_info.json`:
https://raw.githubusercontent.com/jdx/usage/13c051e5f9583804ada6df05bd57471704db8358/LICENSE
Review that pinned supplement when updating Usage. Recursive collection includes
Usage's nested Apache notices/licenses. No license text is invented from an SPDX
name. Missing licenses stop generation for review.

Demand is patched locally; its MIT copyright notice is retained in
vendor/demand/LICENSE and the generated artifact. See vendor/README.md for the
modification. The bitshelf project LICENSE remains separate from third-party
attributions. Archives and Homebrew package share directories carry both the
notices and machine-readable inventory.
