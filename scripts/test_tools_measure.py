"""plan/domains/tools/measure.py finds the commands it measures, so it can't report "0 of 0"."""
import importlib.util
import io
import os
import stat
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

HELP = """usage: cpl <command>

commands:
  check     type-check a file
  build     emit C++
  judges    list judge profiles

options:
  -h, --help
"""


def load(cpl):
    os.environ["CPL"] = cpl
    spec = importlib.util.spec_from_file_location("tools_measure", ROOT / "plan/domains/tools/measure.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def run(fn):
    buf = io.StringIO()
    with redirect_stdout(buf):
        fn()
    return buf.getvalue()


class ToolsMeasureTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        fake = Path(cls.tmp.name) / "cpl"
        fake.write_text("#!/bin/sh\ncat <<'END'\n" + HELP + "END\n")
        fake.chmod(fake.stat().st_mode | stat.S_IEXEC)
        cls.m = load(str(fake))

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_commands_parsed_from_help(self):
        self.assertEqual(self.m.commands(), ["check", "build", "judges"])

    def test_spec_lists_commands(self):
        out = run(self.m.spec_sync)
        spec_only = eval(out.split("spec-only ")[1].split("  cli-only")[0])
        self.assertGreater(len(spec_only), 5, "spec/tools/cli.md yielded no commands; is the path stale?")

    def test_golden_finds_tests(self):
        out = run(self.m.golden)
        counts = dict(zip(out.split()[0::2], out.split()[1::2]))
        self.assertGreater(int(counts["check"]), 0, out)


if __name__ == "__main__":
    unittest.main()
