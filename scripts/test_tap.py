#!/usr/bin/env python3
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('tap', Path(__file__).with_name('prepare-tap.py'))
tap = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tap)


class TapTests(unittest.TestCase):
    def test_published_stable_only(self):
        state = {'tag_name': 'v0.1.0', 'draft': False, 'prerelease': False}
        tap.validate_publication(state, 'v0.1.0')
        for candidate in [None, {}, state | {'draft': True}, state | {'prerelease': True}, state | {'tag_name': 'v0.2.0'}]:
            with self.assertRaises(ValueError):
                tap.validate_publication(candidate, 'v0.1.0')
        with self.assertRaises(ValueError):
            tap.validate_publication(state, 'v0.1.0-rc.1')

    def test_monotonic_updates_and_exact_retries(self):
        current = '  version "0.1.0"\n'
        tap.check_update('', current, 'v0.1.0')
        tap.check_update(current, current, 'v0.1.0')
        tap.check_update(current, '  version "0.2.0"\n', 'v0.2.0')
        for tag in ['v0.0.9', 'v0.1.0']:
            with self.assertRaises(ValueError):
                tap.check_update(current, 'changed', tag)
        with self.assertRaises(ValueError):
            tap.check_update('unrecognized', current, 'v0.1.0')


if __name__ == '__main__':
    unittest.main()
