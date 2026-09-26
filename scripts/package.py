#!/usr/bin/env python3
"""Build a local release archive from a tested binary and generated docs; no upload."""
import argparse
import gzip
import json
import sys
import os
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--target', required=True, choices=['aarch64-apple-darwin', 'x86_64-apple-darwin', 'x86_64-unknown-linux-gnu'])
parser.add_argument('--tag', required=True)
parser.add_argument('--binary', type=Path, required=True)
parser.add_argument('--output-dir', type=Path, default=ROOT / 'dist/archives')
parser.add_argument('--allow-dirty', action='store_true', help='Local preview only; release builds require a clean checkout')
args = parser.parse_args()
if not args.allow_dirty and subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip():
    raise SystemExit('Release packages require a clean checkout; use --allow-dirty only for local previews')
binary = args.binary.resolve()
version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['package']['version']
subprocess.run([sys.executable, str(ROOT / 'scripts/release.py'), 'validate', '--tag', args.tag, '--binary', str(binary)], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
if (ROOT / 'skills/bitshelf/VERSION').read_text().strip() != version:
    raise SystemExit('Skill VERSION must match Cargo')
subprocess.run([sys.executable, str(ROOT / 'scripts/notices.py'), '--check'], cwd=ROOT, check=True)
args.output_dir.mkdir(parents=True, exist_ok=True)
archive = args.output_dir / f'bitshelf-{args.tag}-{args.target}.tar.gz'
epoch = int(subprocess.check_output(['git', 'show', '-s', '--format=%ct', 'HEAD'], cwd=ROOT, text=True))
with tempfile.TemporaryDirectory(prefix='bs-package-') as directory:
    staging = Path(directory)
    shutil.copy2(binary, staging / 'bs')
    (staging / 'bs').chmod(0o755)
    for source, name in [('dist/bs.1', 'bs.1'), ('LICENSE', 'LICENSE'), ('README.md', 'README.md'), ('CHANGELOG.md', 'CHANGELOG.md'), ('THIRD_PARTY_NOTICES.txt', 'THIRD_PARTY_NOTICES.txt'), ('licenses/inventory.json', 'DEPENDENCIES.json')]:
        shutil.copy2(ROOT / source, staging / name)
    shutil.copytree(ROOT / 'dist/completions', staging / 'completions')
    shutil.copytree(ROOT / 'skills', staging / 'skills')
    for source in (ROOT / 'docs').rglob('*'):
        if source.is_file() and '.vitepress' not in source.parts and source.suffix in ['.md', '.json', '.svg']:
            destination = staging / 'docs' / source.relative_to(ROOT / 'docs')
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
    info = subprocess.check_output([sys.executable, str(ROOT / 'scripts/build-info.py'), args.target], cwd=ROOT, text=True)
    (staging / 'BUILD-INFO.json').write_text(info)
    sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip())
    runtime_notices = staging / 'runtime-notices'
    runtime_notices.mkdir()
    shutil.copy2(sysroot / 'share/doc/rust/COPYRIGHT-library.html', runtime_notices)
    shutil.copytree(sysroot / 'share/doc/rust/licenses', runtime_notices / 'licenses')
    with archive.open('wb') as output, gzip.GzipFile(filename='', fileobj=output, mode='wb', mtime=epoch) as compressed, tarfile.open(fileobj=compressed, mode='w') as tar:
        for path in sorted(staging.rglob('*')):
            entry = tar.gettarinfo(str(path), arcname=str(path.relative_to(staging)))
            entry.uid = entry.gid = 0
            entry.uname = entry.gname = ''
            entry.mtime = epoch
            entry.mode = 0o755 if path.is_dir() or path.name == 'bs' else 0o644
            if path.is_file():
                with path.open('rb') as data:
                    tar.addfile(entry, data)
            else:
                tar.addfile(entry)
print(archive.resolve())
