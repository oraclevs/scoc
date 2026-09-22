import tempfile, unittest
from pathlib import Path
from compatibility.sync_jc_inventory import build_inventory

class SyncInventoryTests(unittest.TestCase):
    def test_ast_inventory_collapses_streams_and_defers_windows(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp); (root/'jc/parsers').mkdir(parents=True)
            (root/'jc/lib.py').write_text("parsers = ['ping','ping-s','proc-cpuinfo','dir']\n")
            (root/'jc/parsers/ping.py').write_text("class info:\n version='1.11'\n compatible=['linux','darwin']\n")
            (root/'jc/parsers/ping_s.py').write_text("class info:\n version='1.6'\n compatible=['linux','darwin']\n")
            (root/'jc/parsers/proc_cpuinfo.py').write_text("class info:\n version='1.0'\n compatible=['linux']\n")
            (root/'jc/parsers/dir.py').write_text("class info:\n version='1.0'\n compatible=['win32']\n")
            inv=build_inventory(root)
            self.assertEqual(inv['parsers']['ping']['streaming_name'],'ping-s')
            self.assertEqual(inv['parsers']['dir']['target'],'windows-deferred')
            self.assertIn('proc-cpuinfo',inv['parsers'])
            self.assertNotIn('ping-s',inv['parsers'])
