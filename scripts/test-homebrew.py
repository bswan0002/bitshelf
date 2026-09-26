#!/usr/bin/env python3
"""Install/test/revision-upgrade/uninstall a formula in an ephemeral local tap.

Never pushes a tap or publishes anything. Refuses to disturb existing bitshelf.
"""
import argparse
import os
import shutil
import subprocess
import uuid
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('formula', type=Path)
args = parser.parse_args()
env = os.environ | {'HOMEBREW_NO_AUTO_UPDATE': '1', 'HOMEBREW_NO_INSTALL_CLEANUP': '1', 'HOMEBREW_NO_ANALYTICS': '1', 'HOMEBREW_NO_ENV_HINTS': '1'}
if subprocess.run(['brew', 'list', '--versions', 'bitshelf'], env=env, capture_output=True).returncode == 0:
    raise SystemExit('An existing bitshelf keg is installed; refusing to replace it')
tap = 'bs-local/release-test-' + uuid.uuid4().hex[:10]
full = tap + '/bitshelf'
subprocess.run(['brew', 'tap-new', '--no-git', tap], env=env, check=True)
tap_path = Path(subprocess.check_output(['brew', '--repository', tap], env=env, text=True).strip())
try:
    formula = tap_path / 'Formula/bitshelf.rb'
    formula.parent.mkdir(exist_ok=True)
    shutil.copyfile(args.formula, formula)
    subprocess.run(['brew', 'install', '--formula', full], env=env, check=True)
    subprocess.run(['brew', 'test', full], env=env, check=True)
    prefix = Path(subprocess.check_output(['brew', '--prefix', full], env=env, text=True).strip())
    if not (prefix / 'bin/bs').is_file():
        raise SystemExit('Homebrew did not install bs')
    # A formula revision exercises the package manager's upgrade path without
    # inventing a new CLI version or pretending a second release exists.
    formula.write_text(formula.read_text().replace('  license "MIT"', '  license "MIT"\n  revision 1', 1))
    subprocess.run(['brew', 'upgrade', '--formula', full], env=env, check=True)
    subprocess.run(['brew', 'test', full], env=env, check=True)
    versions = subprocess.check_output(['brew', 'list', '--versions', 'bitshelf'], env=env, text=True)
    if '_1' not in versions:
        raise SystemExit('Formula revision upgrade was not installed')
finally:
    # The unique tap and absence precheck establish ownership of these artifacts.
    installed = subprocess.run(['brew', 'list', '--versions', 'bitshelf'], env=env, capture_output=True).returncode == 0
    if installed:
        subprocess.run(['brew', 'uninstall', '--force', full], env=env, check=True)
    subprocess.run(['brew', 'untap', tap], env=env, check=True)
if subprocess.run(['brew', 'list', '--versions', 'bitshelf'], env=env, capture_output=True).returncode == 0:
    raise SystemExit('Test keg remains after uninstall')
print('Homebrew install/test/revision-upgrade/uninstall passed; local tap removed')
