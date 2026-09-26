#!/usr/bin/env python3
"""Record actual build inputs; runner images are identified, not claimed reproducible."""
import argparse
import hashlib
import json
import os
import platform
import subprocess
from pathlib import Path


def capture(*args):
    return subprocess.check_output(args, text=True).strip()


def parse_macos_minos(otool_output):
    """Return the single minimum macOS version from `otool -l` load commands."""
    versions = set()
    command = None
    for line in otool_output.splitlines():
        fields = line.split()
        if len(fields) == 2 and fields[0] == 'cmd':
            command = fields[1]
        elif len(fields) == 2 and ((command == 'LC_BUILD_VERSION' and fields[0] == 'minos')
                                   or (command == 'LC_VERSION_MIN_MACOSX' and fields[0] == 'version')):
            versions.add(fields[1])
    if len(versions) != 1:
        raise ValueError(f'Expected exactly one macOS minimum version load command, found {sorted(versions)}')
    return versions.pop()


def macos_minos(binary):
    return parse_macos_minos(capture('otool', '-l', str(binary)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('target')
    parser.add_argument('binary', type=Path, help='Exact executable being packaged')
    args = parser.parse_args()
    sysroot = Path(capture('rustc', '--print', 'sysroot'))
    info = {
        'target': args.target,
        'binary_sha256': hashlib.sha256(args.binary.read_bytes()).hexdigest(),
        'rust_runtime_notices_sha256': hashlib.sha256((sysroot / 'share/doc/rust/COPYRIGHT-library.html').read_bytes()).hexdigest(),
        'source_commit': capture('git', 'rev-parse', 'HEAD'),
        'source_dirty': bool(capture('git', 'status', '--porcelain')),
        'rustc': capture('rustc', '--version', '--verbose'),
        'cargo': capture('cargo', '--version'),
        'usage': capture('usage', '--version'),
        'python': platform.python_version(),
        'os': platform.platform(),
        'runner_image': os.environ.get('ImageOS'),
        'runner_image_version': os.environ.get('ImageVersion'),
        'cargo_lock_sha256': hashlib.sha256(Path('Cargo.lock').read_bytes()).hexdigest(),
        'package_lock_sha256': hashlib.sha256(Path('package-lock.json').read_bytes()).hexdigest(),
    }
    if platform.system() == 'Darwin':
        info['sdk'] = capture('xcrun', '--show-sdk-version')
        # The binary's own load command is authoritative; the environment is recorded separately.
        info['macos_minos'] = macos_minos(args.binary)
        info['macosx_deployment_target_env'] = os.environ.get('MACOSX_DEPLOYMENT_TARGET')
    else:
        info['libc'] = platform.libc_ver()
    print(json.dumps(info, indent=2, sort_keys=True))


if __name__ == '__main__':
    main()
