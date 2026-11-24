"""plan/domains/stdlib/measure.py reads the real std/ tree and agrees with the debt ratchet."""
import importlib.util
import io
import subprocess
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("measure", ROOT / "plan/domains/stdlib/measure.py")
measure = importlib.util.module_from_spec(spec)
spec.loader.exec_module(measure)


def run(fn):
    buf = io.StringIO()
    with redirect_stdout(buf):
        fn()
    return buf.getvalue()


class MeasureTest(unittest.TestCase):
    def test_reads_std_modules(self):
        mods = measure.modules()
        self.assertGreater(len(mods), 20, "measure.py found no std modules; is STD stale?")
        for m in ("seq", "graph", "tree", "algebra"):
            self.assertIn(m, mods)

    def test_pub_names_seen(self):
        self.assertIn("SegTree", measure.pub_names())

    def test_todo_split_matches_ratchet(self):
        out = subprocess.run([sys.executable, str(ROOT / "scripts/debt-ratchet.py"), "--show", "stdlib_todo"],
                             capture_output=True, text=True, check=True).stdout.split()
        want = int(out[out.index("stdlib_todo") + 1])
        line = run(measure.todo_split)
        total = int(line.rsplit(",", 1)[1].split()[0])
        self.assertEqual(total, want, line)

    def test_uncovered_counts_modules(self):
        self.assertIn(f"of {len(measure.modules())} modules", run(measure.uncovered))


if __name__ == "__main__":
    unittest.main()
