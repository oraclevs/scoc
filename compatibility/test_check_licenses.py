import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

class LicenseCheckTests(unittest.TestCase):
    def test_repository_provenance_is_closed(self):
        path = ROOT / 'compatibility' / 'check_licenses.py'
        spec = importlib.util.spec_from_file_location('check_licenses', path)
        self.assertIsNotNone(spec)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        self.assertEqual(module.validate(ROOT), [])

if __name__ == '__main__':
    unittest.main()
