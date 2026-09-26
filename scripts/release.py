#!/usr/bin/env python3
"""Release validation and orchestration. Importing this module never publishes."""
import argparse
import json
import re
import subprocess
import tomllib
from pathlib import Path

TAG = re.compile(r'v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-(alpha|beta|rc)\.(0|[1-9]\d*))?')


def metadata(tag, version, changelog, binary=None):
    if not TAG.fullmatch(tag):
        raise ValueError('Use vMAJOR.MINOR.PATCH or vMAJOR.MINOR.PATCH-(alpha|beta|rc).N; no leading zeroes/build metadata')
    if tag != 'v' + version:
        raise ValueError('Tag must match Cargo package version')
    headings = list(re.finditer(r'^##\s+([^\s]+)[^\n]*\n', changelog, re.M))
    matching = [(index, match) for index, match in enumerate(headings) if match[1] == version]
    if len(matching) != 1:
        raise ValueError('Require exactly one matching changelog section')
    index, match = matching[0]
    notes = changelog[match.end():headings[index + 1].start() if index + 1 < len(headings) else len(changelog)].strip()
    if not notes or not re.search(r'^-\s+\S', notes, re.M):
        raise ValueError('Matching changelog section must contain release notes')
    if binary:
        actual = subprocess.check_output([str(Path(binary).resolve()), '--version'], text=True).strip()
        if actual != f'bs {version}':
            raise ValueError(f'Packaged executable version mismatch: {actual!r}')
    return {'tag': tag, 'version': version, 'prerelease': '-' in version, 'notes': notes}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    validate = commands.add_parser('validate')
    validate.add_argument('--tag', required=True)
    validate.add_argument('--binary')
    validate.add_argument('--notes-out', type=Path)
    args = parser.parse_args()
    version = tomllib.loads(Path('Cargo.toml').read_text())['package']['version']
    info = metadata(args.tag, version, Path('CHANGELOG.md').read_text(), args.binary)
    if args.notes_out:
        args.notes_out.write_text(info['notes'] + '\n')
    print(json.dumps(info))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, subprocess.CalledProcessError, OSError) as error:
        raise SystemExit(str(error))
