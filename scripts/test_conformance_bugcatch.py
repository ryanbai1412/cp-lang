import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

spec = importlib.util.spec_from_file_location("bc", Path(__file__).with_name("conformance-bugcatch.py"))
bc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bc)


class RunSuiteTest(unittest.TestCase):
    def test_a_relative_board_is_read_where_the_suite_wrote_it(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / "tree").mkdir()

            def run(cmd, cwd=None, **_):
                if "--scoreboard" in cmd:
                    board = Path(cmd[cmd.index("--scoreboard") + 1])
                    (Path(cwd) / board).write_text(json.dumps({"suites": {"conformance": {"cases": [1]}}}))
                return mock.Mock(returncode=0)

            old = Path.cwd()
            os.chdir(root)
            try:
                with mock.patch.object(bc.subprocess, "run", run):
                    cases = bc.run_suite(root / "tree", {"CARGO_TARGET_DIR": "t"}, Path("base.json"))
            finally:
                os.chdir(old)
            self.assertEqual(cases, [1])
            self.assertTrue((root / "base.json").is_file())


if __name__ == "__main__":
    unittest.main()
