#!/usr/bin/env python3
import importlib.util
import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('release', Path(__file__).with_name('release.py'))
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class MetadataTests(unittest.TestCase):
    notes = '# Changelog\n\n## 0.1.0 — Prepared\n\n- Feature.\n\n## 0.0.1\n\n- Old.\n'

    def test_matching_version_and_exact_notes(self):
        result = release.metadata('v0.1.0', '0.1.0', self.notes)
        self.assertEqual(result['notes'], '- Feature.')
        self.assertFalse(result['prerelease'])

    def test_invalid_versions_channels_and_notes(self):
        for tag in ['v0.2.0', 'v01.1.0', '0.1.0', 'v0.1.0+build', 'v0.1.0-rc', 'v0.1.0-rc.01']:
            with self.assertRaises(ValueError):
                release.metadata(tag, '0.1.0', self.notes)
        for notes in ['', '## 0.2.0\n- Other', '## 0.1.0\n\n', '## 0.1.0\n- One\n## 0.1.0\n- Two']:
            with self.assertRaises(ValueError):
                release.metadata('v0.1.0', '0.1.0', notes)

    def test_prerelease_and_final_executable(self):
        result = release.metadata('v0.1.0-rc.1', '0.1.0-rc.1', '## 0.1.0-rc.1\n- Candidate')
        self.assertTrue(result['prerelease'])
        with patch.object(release.subprocess, 'check_output', return_value='bs 0.2.0\n'):
            with self.assertRaises(ValueError):
                release.metadata('v0.1.0', '0.1.0', self.notes, '/tmp/bs')
        with patch.object(release.subprocess, 'check_output', return_value='bs 0.1.0\n'):
            release.metadata('v0.1.0', '0.1.0', self.notes, '/tmp/bs')


class FakeGitHub:
    def __init__(self, state=None):
        self.state = state
        self.assets = {}
        self.calls = []

    def __call__(self, *args):
        args = [str(a) for a in args]
        self.calls.append(args)
        if args[0] == 'api':
            if self.state is not None:
                self.state['assets'] = [{'name': name} for name in self.assets]
            return json.dumps([[self.state] if self.state else []])
        action = args[1]
        if action == 'create':
            self.state = {'tag_name': args[2], 'draft': True}
        elif action == 'upload':
            path = Path(args[3])
            if path.name in self.assets:
                raise ValueError('Duplicate upload')
            self.assets[path.name] = path.read_bytes()
        elif action == 'download':
            directory = Path(args[args.index('--dir') + 1])
            names = [args[args.index('--pattern') + 1]] if '--pattern' in args else self.assets.keys()
            for name in names:
                (directory / name).write_bytes(self.assets[name])
        elif action == 'edit':
            self.state['draft'] = False
        else:
            raise AssertionError(args)
        return ''


class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.info = release.metadata('v0.1.0', '0.1.0', MetadataTests.notes)
        checksums = []
        for name in release.archive_names('v0.1.0'):
            content = name.encode()
            (self.directory / name).write_bytes(content)
            checksums.append(hashlib.sha256(content).hexdigest() + '  ' + name)
        (self.directory / 'SHA256SUMS').write_text('\n'.join(checksums) + '\n')

    def test_draft_publish_and_matching_retry_never_replace_assets(self):
        fake = FakeGitHub({'tag_name': 'v0.1.0', 'draft': True})
        first = release.archive_names('v0.1.0')[0]
        fake.assets[first] = (self.directory / first).read_bytes()
        with patch.object(release, 'gh', fake):
            release.upload_and_publish('owner/repo', self.info, self.directory)
        self.assertFalse(fake.state['draft'])
        self.assertFalse(any('--clobber' in args for args in fake.calls))
        self.assertEqual(sum(args[:2] == ['release', 'upload'] for args in fake.calls), 3)

    def test_new_release_creation_and_prerelease_classification(self):
        fake = FakeGitHub()
        with patch.object(release, 'gh', fake):
            release.upload_and_publish('owner/repo', self.info, self.directory)
        self.assertTrue(any(args[:2] == ['release', 'create'] for args in fake.calls))
        self.assertIn('--prerelease=false', fake.calls[-1])

    def test_published_and_api_failure_do_not_upload_or_create(self):
        fake = FakeGitHub({'tag_name': 'v0.1.0', 'draft': False})
        with patch.object(release, 'gh', fake), self.assertRaises(ValueError):
            release.upload_and_publish('owner/repo', self.info, self.directory)
        self.assertEqual(len(fake.calls), 1)
        with patch.object(release, 'gh', side_effect=subprocess.CalledProcessError(1, 'gh')) as mock:
            with self.assertRaises(subprocess.CalledProcessError):
                release.upload_and_publish('owner/repo', self.info, self.directory)
            self.assertEqual(mock.call_count, 1)

    def test_different_draft_assets_require_explicit_recovery(self):
        fake = FakeGitHub({'tag_name': 'v0.1.0', 'draft': True})
        fake.assets[release.archive_names('v0.1.0')[0]] = b'different'
        with patch.object(release, 'gh', fake), self.assertRaisesRegex(ValueError, 'differs'):
            release.upload_and_publish('owner/repo', self.info, self.directory)
        self.assertTrue(fake.state['draft'])
        self.assertFalse(any(args[:2] == ['release', 'upload'] for args in fake.calls))

    def test_incomplete_and_corrupt_local_artifacts_fail_before_network(self):
        name = release.archive_names('v0.1.0')[0]
        for operation in ['corrupt', 'remove']:
            if operation == 'corrupt':
                (self.directory / name).write_bytes(b'bad')
            else:
                (self.directory / name).unlink()
            with patch.object(release, 'gh') as mock, self.assertRaises(ValueError):
                release.upload_and_publish('owner/repo', self.info, self.directory)
            mock.assert_not_called()


if __name__ == '__main__':
    unittest.main()
