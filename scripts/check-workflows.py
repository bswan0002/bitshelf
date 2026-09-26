#!/usr/bin/env python3
"""Run pinned upstream actionlint in a temporary directory, checking its checksum."""
import hashlib
import platform
import subprocess
import tarfile
import tempfile
import urllib.request
from pathlib import Path

version = '1.7.12'
os_name = {'Darwin': 'darwin', 'Linux': 'linux'}[platform.system()]
arch = {'arm64': 'arm64', 'aarch64': 'arm64', 'x86_64': 'amd64'}[platform.machine()]
name = f'actionlint_{version}_{os_name}_{arch}.tar.gz'
base = f'https://github.com/rhysd/actionlint/releases/download/v{version}/'
checksums = urllib.request.urlopen(base + f'actionlint_{version}_checksums.txt').read().decode()
expected = next(line.split()[0] for line in checksums.splitlines() if line.split()[-1] == name)
with tempfile.TemporaryDirectory(prefix='bs-actionlint-') as directory:
    archive = Path(directory) / name
    archive.write_bytes(urllib.request.urlopen(base + name).read())
    if hashlib.sha256(archive.read_bytes()).hexdigest() != expected:
        raise SystemExit('actionlint checksum mismatch')
    with tarfile.open(archive) as tar:
        tar.extractall(directory, filter='data')
    subprocess.run([str(Path(directory) / 'actionlint'), '-color'], check=True, cwd=Path(__file__).resolve().parent.parent)
