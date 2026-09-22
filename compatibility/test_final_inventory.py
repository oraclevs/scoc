from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]

class FinalInventoryTests(unittest.TestCase):
    def test_jc_inventory_is_closed(self):
        expected = set((ROOT/'compatibility/expected-first-class-jc.txt').read_text().splitlines())
        data = tomllib.loads((ROOT/'compatibility/jc-inventory.toml').read_text())
        parsers = data['parsers']
        registered = {name for name, meta in parsers.items() if meta['target'] != 'windows-deferred'}
        deferred = {name for name, meta in parsers.items() if meta['target'] == 'windows-deferred'}
        self.assertEqual(expected, registered)
        self.assertEqual({'dir','ipconfig','net-localgroup','net-user','route-print','systeminfo'}, deferred)
        self.assertEqual(223, len(parsers))

    def test_native_catalog_has_frozen_57(self):
        expected = set((ROOT/'compatibility/expected-native.txt').read_text().splitlines())
        data = tomllib.loads((ROOT/'compatibility/native-catalog.toml').read_text())
        self.assertEqual(expected, set(data['parsers']))
        self.assertEqual(57, len(data['parsers']))

    def test_all_registered_targets_have_modules(self):
        jc = set((ROOT/'compatibility/expected-first-class-jc.txt').read_text().splitlines())
        native = set((ROOT/'compatibility/expected-native.txt').read_text().splitlines())
        jc_files = {p.stem.replace('_','-') for p in (ROOT/'src/parsers/jc').glob('*.rs') if p.name != 'mod.rs'}
        native_files = {p.stem.replace('_','-') for p in (ROOT/'src/parsers/native').glob('*/*.rs') if p.name != 'mod.rs'}
        self.assertTrue(jc <= jc_files, sorted(jc-jc_files))
        self.assertTrue(native <= native_files, sorted(native-native_files))

if __name__ == '__main__':
    unittest.main()
