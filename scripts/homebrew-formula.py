#!/usr/bin/env python3
"""Generate the Homebrew formula from verified archive checksums; never publish."""
import argparse
import re
from pathlib import Path
from urllib.parse import urlsplit

TARGETS = ('aarch64-apple-darwin', 'x86_64-apple-darwin', 'x86_64-unknown-linux-gnu')


def generate(tag, checksum_text, base_url=None, local_test=False):
    if not re.fullmatch(r'v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)', tag):
        raise ValueError('Homebrew requires a stable vMAJOR.MINOR.PATCH tag')
    expected = {f'bitshelf-{tag}-{target}.tar.gz' for target in TARGETS}
    checksums = {}
    for line in checksum_text.splitlines():
        digest, filename = line.split()
        if filename in checksums or filename not in expected or not re.fullmatch(r'[0-9a-f]{64}', digest):
            raise ValueError('Invalid, duplicate or unexpected checksum entry')
        checksums[filename] = digest
    if not checksums or (not local_test and set(checksums) != expected):
        raise ValueError('All target checksums are required for the production formula')
    if local_test and (not base_url or urlsplit(base_url).scheme != 'file'):
        raise ValueError('Partial local testing requires an explicit file:// archive directory')
    base_url = base_url or f'https://github.com/bswan0002/bitshelf/releases/download/{tag}'
    if urlsplit(base_url).scheme not in ('file', 'https') or any(c in base_url for c in ['"', "'", '\n', '\r', '#', '\\']):
        raise ValueError('Unsafe archive base URL')

    def artifact(target):
        filename = f'bitshelf-{tag}-{target}.tar.gz'
        if filename not in checksums:
            return 'odie "This local test has no artifact for this architecture"'
        return f'url "{base_url.rstrip("/")}/{filename}"\n      sha256 "{checksums[filename]}"'

    return f'''class Bitshelf < Formula
  desc "Local Markdown-first storage for reusable bits"
  homepage "https://github.com/bswan0002/bitshelf"
  version "{tag[1:]}"
  license "MIT"

  on_macos do
    depends_on macos: :sequoia
    if Hardware::CPU.arm?
      {artifact("aarch64-apple-darwin")}
    else
      {artifact("x86_64-apple-darwin")}
    end
  end

  on_linux do
    depends_on arch: :x86_64
    {artifact("x86_64-unknown-linux-gnu")}
  end

  def install
    bin.install "bs"
    man1.install "bs.1"
    bash_completion.install "completions/bs.bash" => "bs"
    zsh_completion.install "completions/_bs"
    fish_completion.install "completions/bs.fish"
    pkgshare.install "skills", "docs", "CHANGELOG.md", "runtime-notices"
    pkgshare.install "THIRD_PARTY_NOTICES.txt", "DEPENDENCIES.json", "BUILD-INFO.json"
  end

  test do
    config = testpath/"config.toml"
    body = testpath/"body.txt"
    body.write "exact\\r\\nbody without final newline"
    system bin/"bs", "--config", config, "init", "--store", testpath/"store"
    system bin/"bs", "--config", config, "add", "notes/brew-test", "--file", body
    assert_equal body.read, Utils.safe_popen_read(bin/"bs", "--config", config, "show", "notes/brew-test", "--body")
    system bin/"bs", "--config", config, "edit", "notes/brew-test", "--title", "Homebrew"
    result = JSON.parse(Utils.safe_popen_read(bin/"bs", "--config", config, "list", "--json"))
    assert_equal true, result.fetch("complete")
    assert_equal "Homebrew", result.fetch("results").first.fetch("title")
    assert_equal "bs {tag[1:]}", Utils.safe_popen_read(bin/"bs", "--version").strip
    assert_equal "{tag[1:]}", (pkgshare/"skills/bitshelf/VERSION").read.strip
    assert_predicate pkgshare/"THIRD_PARTY_NOTICES.txt", :file?
    assert_predicate pkgshare/"DEPENDENCIES.json", :file?
    assert_predicate pkgshare/"runtime-notices/COPYRIGHT-library.html", :file?
    assert_predicate pkgshare/"BUILD-INFO.json", :file?
    assert_predicate pkgshare/"docs/json.md", :file?
    assert_predicate man1/"bs.1", :file?
    assert_predicate bash_completion/"bs", :file?
    assert_predicate zsh_completion/"_bs", :file?
    assert_predicate fish_completion/"bs.fish", :file?
  end
end
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('tag')
    parser.add_argument('checksum_file', type=Path)
    parser.add_argument('--base-url')
    parser.add_argument('--local-test', action='store_true')
    args = parser.parse_args()
    print(generate(args.tag, args.checksum_file.read_text(), args.base_url, args.local_test), end='')


if __name__ == '__main__':
    try:
        main()
    except ValueError as error:
        raise SystemExit(str(error))
