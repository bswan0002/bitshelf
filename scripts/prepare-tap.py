#!/usr/bin/env python3
"""Prepare a stable formula from an already-published release. Never push."""
import argparse
import importlib.util
import re
import subprocess
import tempfile
from pathlib import Path
import release

spec = importlib.util.spec_from_file_location('formula', Path(__file__).with_name('homebrew-formula.py'))
formula = importlib.util.module_from_spec(spec)
spec.loader.exec_module(formula)


def validate_publication(state, tag):
    if not re.fullmatch(r'v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)', tag):
        raise ValueError('Only stable release tags can update the tap')
    if not state or state.get('draft') is not False or state.get('prerelease') is not False or state.get('tag_name') != tag:
        raise ValueError('Require an existing published stable release')


def check_update(current, candidate, tag):
    if current == candidate or not current:
        return
    match = re.search(r'^  version "(\d+)\.(\d+)\.(\d+)"$', current, re.M)
    if not match:
        raise ValueError('Existing formula version is unrecognized; inspect manually')
    old = tuple(map(int, match.groups()))
    new = tuple(map(int, tag[1:].split('.')))
    if new <= old:
        raise ValueError('Refusing downgrade or different formula bytes for the same version')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tag', required=True)
    parser.add_argument('--repo', default='bswan0002/bitshelf')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', args.repo):
        raise ValueError('Invalid repository name')
    validate_publication(release.release_state(args.repo, args.tag), args.tag)
    with tempfile.TemporaryDirectory(prefix='bs-tap-') as work:
        release.gh('release', 'download', args.tag, '--repo', args.repo, '--dir', work)
        release.verify_artifacts(work, args.tag)
        candidate = formula.generate(args.tag, (Path(work) / 'SHA256SUMS').read_text(),
                                     f'https://github.com/{args.repo}/releases/download/{args.tag}')
    check_update(args.output.read_text() if args.output.exists() else '', candidate, args.tag)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(candidate)
    print(args.output)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error))
