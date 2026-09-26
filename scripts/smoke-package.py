#!/usr/bin/env python3
"""Exercise an extracted archive without checkout dependencies or user configuration."""
import argparse
import hashlib
import json
import os
import subprocess
import tarfile
import tempfile
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('archive', type=Path)
parser.add_argument('--tag', required=True)
parser.add_argument('--target', required=True)
args = parser.parse_args()
with tempfile.TemporaryDirectory(prefix='bs-package-smoke-') as directory:
    base = Path(directory)
    package = base / 'package'
    package.mkdir()
    with tarfile.open(args.archive) as tar:
        for entry in tar.getmembers():
            name = Path(entry.name)
            if name.is_absolute() or '..' in name.parts or not (entry.isfile() or entry.isdir()):
                raise SystemExit(f'Unsafe archive entry: {entry.name}')
        tar.extractall(package, filter='data')
    required = ['bs', 'bs.1', 'LICENSE', 'README.md', 'CHANGELOG.md', 'THIRD_PARTY_NOTICES.txt', 'DEPENDENCIES.json', 'BUILD-INFO.json', 'completions/bs.bash', 'completions/_bs', 'completions/bs.fish', 'skills/bitshelf/SKILL.md', 'skills/bitshelf/VERSION', 'docs/json.md', 'docs/concepts/timestamps.md', 'runtime-notices/COPYRIGHT-library.html']
    for name in required:
        if not (package / name).is_file() or not (package / name).stat().st_size:
            raise SystemExit(f'Missing/empty package file: {name}')
    if not os.access(package / 'bs', os.X_OK):
        raise SystemExit('Packaged bs is not executable')
    info = json.loads((package / 'BUILD-INFO.json').read_text())
    if info['target'] != args.target:
        raise SystemExit('Build-info target mismatch')
    assert hashlib.sha256((package / 'runtime-notices/COPYRIGHT-library.html').read_bytes()).hexdigest() == info['rust_runtime_notices_sha256']
    architecture = subprocess.check_output(['/usr/bin/file', str(package / 'bs')], text=True)
    expected = 'arm64' if args.target == 'aarch64-apple-darwin' else 'x86_64' if args.target.endswith('apple-darwin') else 'x86-64'
    if expected not in architecture:
        raise SystemExit(f'Unexpected binary architecture: {architecture}')
    if (package / 'skills/bitshelf/VERSION').read_text().strip() != args.tag[1:]:
        raise SystemExit('Skill version mismatch')
    inventory = json.loads((package / 'DEPENDENCIES.json').read_text())
    names = {item['name'] for item in inventory['packages']}
    if not {'demand', 'serde-saphyr', 'usage-rs'} <= names:
        raise SystemExit('Incomplete dependency inventory')
    if 'Copyright (c) 2023 jdx' not in (package / 'THIRD_PARTY_NOTICES.txt').read_text():
        raise SystemExit('Missing vendored Demand copyright')
    env = {'HOME': str(base), 'PATH': '/usr/bin:/bin:/usr/sbin:/sbin', 'LANG': 'en_US.UTF-8'}
    config = base / 'config.toml'
    def run(*command):
        return subprocess.check_output([str(package / 'bs'), '--config', str(config), *map(str, command)], env=env, cwd=base)
    assert run('--version').decode().strip() == f'bs {args.tag[1:]}'
    run('init', '--store', base / 'store')
    body = b'exact\r\nbody without final newline'
    body_file = base / 'body'
    body_file.write_bytes(body)
    run('add', 'notes/package-test', '--file', body_file, '--set-json', 'custom={"flag":true}')
    assert run('show', 'notes/package-test', '--body') == body
    run('edit', 'notes/package-test', '--title', 'Packaged')
    rows = json.loads(run('list', '--json'))
    assert rows['complete'] is True and rows['errors'] == []
    assert rows['results'][0]['title'] == 'Packaged'
    assert rows['results'][0]['metadata']['custom'] == {'flag': True}
    run('shelf', 'add', 'archive')
    run('move', 'notes/package-test', 'archive', '--tags', 'package:test')
    assert run('show', 'archive/package-test', '--body') == body
    run('validate')
    for shell in ['bash', 'zsh', 'fish']:
        assert run('completion', '--shell', shell)
    # Generated shell files and man page must be nonempty and reference bs.
    for name in ['bs.1', 'completions/bs.bash', 'completions/_bs', 'completions/bs.fish']:
        assert 'bs' in (package / name).read_text()
    assert not (base / 'store/.bitshelf/state.json').exists()
print(f'Extracted package smoke passed: {args.target} {args.tag}')
