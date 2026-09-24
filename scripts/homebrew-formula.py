#!/usr/bin/env python3
"""Generate a formula only from checksums of successfully built release archives."""
import re
import sys
from pathlib import Path

tag, checksum_file = sys.argv[1:]
if not re.fullmatch(r"v\d+\.\d+\.\d+", tag):
    raise SystemExit("Homebrew publishing requires a stable vMAJOR.MINOR.PATCH tag")
checksums = {}
for line in Path(checksum_file).read_text().splitlines():
    digest, filename = line.split()
    if not re.fullmatch(r"[0-9a-f]{64}", digest):
        raise SystemExit("Invalid SHA256")
    checksums[filename.lstrip("*")] = digest


def artifact(target):
    filename = f"bitshelf-{tag}-{target}.tar.gz"
    return f'''url "https://github.com/bswan0002/bitshelf/releases/download/{tag}/{filename}"
      sha256 "{checksums[filename]}"'''


print(f'''class Bitshelf < Formula
  desc "Local Markdown-first storage for reusable bits"
  homepage "https://github.com/bswan0002/bitshelf"
  version "{tag[1:]}"
  license "MIT"

  on_macos do
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
    pkgshare.install "skills"
  end

  test do
    config = testpath/"config.toml"
    system bin/"bs", "--config", config, "init", "--store", testpath/"store"
    system bin/"bs", "--config", config, "add", "notes", "--title", "Brew test"
    assert_match "notes/brew-test", shell_output("#{{bin}}/bs --config #{{config}} list --json")
  end
end''')
