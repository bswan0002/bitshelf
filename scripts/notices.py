#!/usr/bin/env python3
"""Generate deterministic notices for the locked non-dev dependency graph.

Includes build dependencies and all target branches conservatively. License texts
come from the exact Cargo package sources, including local dependency patches.
"""
import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def generate():
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1'], cwd=ROOT, text=True))
    packages = {p['id']: p for p in metadata['packages']}
    root = metadata['resolve']['root']
    identities = set()
    for target in ['aarch64-apple-darwin', 'x86_64-apple-darwin', 'x86_64-unknown-linux-gnu']:
        tree = subprocess.check_output(['cargo', 'tree', '--locked', '--target', target, '-e', 'normal,build', '--prefix', 'none', '--format', '{p}'], cwd=ROOT, text=True)
        for line in tree.splitlines():
            match = re.match(r'^(\S+) v(\S+)', line)
            if not match:
                raise ValueError(f'Unexpected dependency tree row: {line}')
            identities.add((match[1], match[2]))
    seen = {identifier for identifier, package in packages.items() if (package['name'], package['version']) in identities}
    inventory = {'cargo_lock_sha256': hashlib.sha256((ROOT / 'Cargo.lock').read_bytes()).hexdigest(), 'scope': 'Non-dev graph, including build dependencies across supported release targets; may conservatively include code absent from a particular binary.', 'packages': []}
    sections = ['THIRD-PARTY NOTICES\n\nProject bitshelf is MIT licensed; its LICENSE is separate.\nThis file reproduces notices from exact locked dependency sources.\nLocally patched Demand and usage-argv sources retain their upstream licenses.\n']
    for package in sorted((packages[i] for i in seen if i != root), key=lambda p: (p['name'], p['version'])):
        directory = Path(package['manifest_path']).parent
        files = set()
        for entry in directory.rglob("*"):
            name = entry.name.lower()
            if name.startswith(('license', 'licence', 'copying', 'notice', 'copyright', 'unlicense')):
                if entry.is_file():
                    files.add(entry)
                elif entry.is_dir():
                    files.update(p for p in entry.rglob('*') if p.is_file())
        if package.get('license_file'):
            files.add(directory / package['license_file'])
        supplemental = []
        if package['name'].startswith('usage-') and package['version'] == '6.11.1':
            supplemental.append(ROOT / 'licenses/upstream/usage-6.11.1-LICENSE')
        if not files and not supplemental:
            raise ValueError(f"Missing license texts for {package['name']} {package['version']}; review its packaged sources before distributing")
        record = {'name': package['name'], 'version': package['version'], 'license': package['license'], 'repository': package['repository'], 'source': 'local patched source' if package['source'] is None else package['source'], 'files': []}
        sections.append(f"\n{'=' * 72}\n{package['name']} {package['version']}\nLicense expression: {package['license']}\nRepository: {package['repository']}\n")
        for path in sorted(files) + supplemental:
            text = path.read_text(encoding='utf-8')
            relative = str(path.relative_to(directory)) if path.is_relative_to(directory) else str(path.relative_to(ROOT))
            record['files'].append({'path': relative, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
            sections.append(f'\n--- {relative} ---\n{text.rstrip()}\n')
        inventory['packages'].append(record)
    return ''.join(sections), json.dumps(inventory, indent=2, sort_keys=True) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    notices, inventory = generate()
    for path, content in [(ROOT / 'THIRD_PARTY_NOTICES.txt', notices), (ROOT / 'licenses' / 'inventory.json', inventory)]:
        if args.check:
            if not path.exists() or path.read_text() != content:
                raise SystemExit(f'Stale notices: {path}; run python3.12 scripts/notices.py and review')
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)


if __name__ == '__main__':
    main()
