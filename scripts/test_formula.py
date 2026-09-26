#!/usr/bin/env python3
import hashlib
import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location('formula', Path(__file__).with_name('homebrew-formula.py'))
formula = importlib.util.module_from_spec(spec)
spec.loader.exec_module(formula)


class FormulaTests(unittest.TestCase):
    def setUp(self):
        self.sums = '\n'.join(hashlib.sha256(target.encode()).hexdigest() + f'  bitshelf-v0.1.0-{target}.tar.gz' for target in formula.TARGETS)

    def test_production_has_exact_targets_and_installed_contract_tests(self):
        value = formula.generate('v0.1.0', self.sums)
        for target in formula.TARGETS:
            self.assertIn(f'/v0.1.0/bitshelf-v0.1.0-{target}.tar.gz', value)
            self.assertIn(hashlib.sha256(target.encode()).hexdigest(), value)
        for text in ['Hardware::CPU.arm?', 'on_linux', 'THIRD_PARTY_NOTICES.txt', 'VERSION', 'test do']:
            self.assertIn(text, value)

    def test_invalid_or_incomplete_production_inputs_fail(self):
        for sums in ['', self.sums.splitlines()[0], self.sums + '\n' + self.sums.splitlines()[0], self.sums.replace('bitshelf-', '../bitshelf-')]:
            with self.assertRaises(ValueError):
                formula.generate('v0.1.0', sums)
        for tag in ['v0.1.0-rc.1', 'v01.1.0', 'v0.1.0+foo']:
            with self.assertRaises(ValueError):
                formula.generate(tag, self.sums)

    def test_local_partial_artifact_requires_explicit_local_mode(self):
        sums = self.sums.splitlines()[0]
        with self.assertRaises(ValueError):
            formula.generate('v0.1.0', sums, 'https://example.com', True)
        result = formula.generate('v0.1.0', sums, 'file:///tmp/archives', True)
        self.assertIn('file:///tmp/archives/bitshelf-v0.1.0-aarch64-apple-darwin.tar.gz', result)
        self.assertIn('This local test has no artifact', result)
        with self.assertRaises(ValueError):
            formula.generate('v0.1.0', self.sums, 'https://example.com/";bad')


if __name__ == '__main__':
    unittest.main()
