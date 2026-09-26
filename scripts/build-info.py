#!/usr/bin/env python3
"""Record actual build inputs; runner images are identified, not claimed reproducible."""
import hashlib
import json
import os
import platform
import subprocess
import sys
from pathlib import Path


def capture(*args):
    return subprocess.check_output(args, text=True).strip()


sysroot = Path(capture('rustc', '--print', 'sysroot'))
info = {
    'target': sys.argv[1],
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
    'deployment_target': os.environ.get('MACOSX_DEPLOYMENT_TARGET'),
    'cargo_lock_sha256': hashlib.sha256(Path('Cargo.lock').read_bytes()).hexdigest(),
    'package_lock_sha256': hashlib.sha256(Path('package-lock.json').read_bytes()).hexdigest(),
}
if platform.system() == 'Darwin':
    info['sdk'] = capture('xcrun', '--show-sdk-version')
else:
    info['libc'] = platform.libc_ver()
print(json.dumps(info, indent=2, sort_keys=True))
