#!/usr/bin/env python3
"""Release validation and orchestration. Importing this module never publishes."""
import argparse
import hashlib
import tempfile
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


TARGETS = ('aarch64-apple-darwin', 'x86_64-apple-darwin', 'x86_64-unknown-linux-gnu')


def gh(*args):
    return subprocess.check_output(['gh', *map(str, args)], text=True)


def release_state(repo, tag):
    # An API/authentication error is never evidence that a release is absent.
    pages = json.loads(gh('api', '--paginate', '--slurp', f'repos/{repo}/releases'))
    if not isinstance(pages, list) or any(not isinstance(page, list) for page in pages):
        raise ValueError('Malformed release inventory')
    matches = [item for page in pages for item in page if item.get('tag_name') == tag]
    if len(matches) > 1:
        raise ValueError('Ambiguous release inventory')
    return matches[0] if matches else None


def require_unpublished(repo, tag):
    state = release_state(repo, tag)
    if state is not None and state.get('draft') is not True:
        raise ValueError('Release is already published; artifacts are immutable. Use the tap-only recovery workflow.')
    return state


def archive_names(tag):
    return [f'bitshelf-{tag}-{target}.tar.gz' for target in TARGETS]


def verify_artifacts(directory, tag):
    directory = Path(directory)
    expected = set(archive_names(tag))
    found = {p.name for p in directory.glob('*.tar.gz')}
    if found != expected:
        raise ValueError(f'Archive set mismatch: expected {sorted(expected)}, found {sorted(found)}')
    sums = {}
    for line in (directory / 'SHA256SUMS').read_text().splitlines():
        digest, filename = line.split()
        if filename in sums or filename not in expected or not re.fullmatch(r'[0-9a-f]{64}', digest):
            raise ValueError('Invalid or duplicate checksum entry')
        sums[filename] = digest
    if set(sums) != expected:
        raise ValueError('Checksum set is incomplete')
    for filename, digest in sums.items():
        if hashlib.sha256((directory / filename).read_bytes()).hexdigest() != digest:
            raise ValueError(f'Checksum mismatch: {filename}')
    return sorted(expected) + ['SHA256SUMS']


def upload_and_publish(repo, info, directory):
    names = verify_artifacts(directory, info['tag'])
    state = require_unpublished(repo, info['tag'])
    with tempfile.TemporaryDirectory(prefix='bs-release-') as work:
        notes = Path(work) / 'notes.md'
        notes.write_text(info['notes'] + '\n')
        if state is None:
            options = ['--prerelease'] if info['prerelease'] else []
            gh('release', 'create', info['tag'], '--repo', repo, '--verify-tag', '--draft', '--title', info['tag'], '--notes-file', notes, *options)
        for filename in names:
            state = require_unpublished(repo, info['tag'])
            if state is None:
                raise ValueError('Draft disappeared during upload')
            existing = [a for a in state.get('assets', []) if a.get('name') == filename]
            if existing:
                # Safe retry: reuse exactly matching bytes, never --clobber.
                download = Path(work) / filename
                gh('release', 'download', info['tag'], '--repo', repo, '--pattern', filename, '--dir', work)
                if download.read_bytes() != (Path(directory) / filename).read_bytes():
                    raise ValueError(f'Draft asset differs: {filename}; inspect/delete the unpublished draft before rebuilding')
            else:
                gh('release', 'upload', info['tag'], Path(directory) / filename, '--repo', repo)
        state = require_unpublished(repo, info['tag'])
        if state is None or {a.get('name') for a in state.get('assets', [])} != set(names):
            raise ValueError('Remote draft artifact set is incomplete or contains unexpected assets')
        # Validate remotely uploaded bytes, not just local input.
        downloaded = Path(work) / 'verified'
        downloaded.mkdir()
        gh('release', 'download', info['tag'], '--repo', repo, '--dir', downloaded)
        verify_artifacts(downloaded, info['tag'])
        require_unpublished(repo, info['tag'])
        gh('release', 'edit', info['tag'], '--repo', repo, '--notes-file', notes,
           '--prerelease=' + str(info['prerelease']).lower(), '--draft=false')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    for command in ['validate', 'preflight', 'publish', 'verify-artifacts']:
        sub = commands.add_parser(command)
        sub.add_argument('--tag', required=True)
        sub.add_argument('--binary')
        sub.add_argument('--notes-out', type=Path)
        if command in ['preflight', 'publish']:
            sub.add_argument('--repo', required=True)
        if command in ['publish', 'verify-artifacts']:
            sub.add_argument('--directory', type=Path, required=True)
    args = parser.parse_args()
    version = tomllib.loads(Path('Cargo.toml').read_text())['package']['version']
    info = metadata(args.tag, version, Path('CHANGELOG.md').read_text(), args.binary)
    if args.notes_out:
        args.notes_out.write_text(info['notes'] + '\n')
    if args.command == 'preflight':
        require_unpublished(args.repo, args.tag)
    elif args.command == 'publish':
        upload_and_publish(args.repo, info, args.directory)
    elif args.command == 'verify-artifacts':
        verify_artifacts(args.directory, args.tag)
    print(json.dumps(info))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, subprocess.CalledProcessError, OSError) as error:
        raise SystemExit(str(error))
