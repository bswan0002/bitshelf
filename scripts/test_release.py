#!/usr/bin/env python3
import importlib.util
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


if __name__ == '__main__':
    unittest.main()
