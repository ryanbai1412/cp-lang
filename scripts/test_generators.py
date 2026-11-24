import importlib.util
import os
import sys
import unittest
from pathlib import Path

_CORPUS = Path(__file__).resolve().parent.parent / "corpus"
sys.path.insert(0, str(_CORPUS))
_SPEC = importlib.util.spec_from_file_location("generators", _CORPUS / "generators.py")
gn = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(gn)


class AtMax(unittest.TestCase):
    """`at_max` on scripted runs: the checked C++ first, then three judge runs, then three accepted runs."""

    def findings(self, safe, mine, theirs=(0, b"1\n", 0.05, 10), limits=(2000, 256)):
        results = iter([safe, mine, mine, mine, theirs, theirs, theirs])
        for name, fake in (("measure", lambda exe, data, t: next(results)), ("checked", lambda *a: "safe"),
                           ("limits", lambda port: limits), ("agrees", lambda d, data, got, want: got == want)):
            self.addCleanup(setattr, gn, name, getattr(gn, name))
            setattr(gn, name, fake)
        return gn.at_max("p", "d", "port.cp", "cpl", "exe", "orig", b"")

    def test_a_fast_matching_run_is_no_finding(self):
        self.assertEqual(self.findings((0, b"1\n", 0.1, 10), (0, b"1\n", 0.05, 10)), [])

    def test_a_checked_trap_is_a_finding(self):
        self.assertEqual(self.findings((3, b"", 0.1, 10), (0, b"1\n", 0.05, 10)), ["traps p --max"])

    def test_over_the_time_limit(self):
        self.assertIn("over-limit p --max", self.findings((0, b"1\n", 1, 10), (0, b"1\n", 2.5, 10), (0, b"1\n", 2.4, 10)))

    def test_over_the_memory_limit(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), (0, b"1\n", 0.05, 300)), ["over-limit p --max"])

    def test_a_judge_timeout_is_over_limit(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), None), ["over-limit p --max"])

    def test_a_different_answer_disagrees(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), (0, b"2\n", 0.05, 10)), ["disagrees p --max"])

    def test_slow_against_the_accepted_cpp(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), (0, b"1\n", 0.5, 10), (0, b"1\n", 0.2, 10)), ["slow p --max"])


class Limits(unittest.TestCase):
    def test_the_limits_line_and_the_default(self):
        path = Path(os.environ.get("TMPDIR", "/tmp")) / "test_generators_port.cp"
        path.write_text("// Source: x\n// limits: 3000 ms, 512 MB\nlet n: 1..=5 = read()\n")
        self.addCleanup(path.unlink)
        self.assertEqual(gn.limits(str(path)), (3000, 512))
        path.write_text("let n: 1..=5 = read()\n")
        self.assertEqual(gn.limits(str(path)), (2000, 256))


class Measure(unittest.TestCase):
    """`measure` reports the program's own peak memory, however much the gate holds."""

    def test_a_small_program_is_small_while_the_gate_is_big(self):
        import shutil
        big = bytearray(300 << 20)
        for i in range(0, len(big), 4096):
            big[i] = 1
        code, out, _, mb = gn.measure(shutil.which("cat"), b"hi\n", 20)
        self.assertEqual((code, out), (0, b"hi\n"))
        self.assertLess(mb, 50)
        del big

    def test_a_timeout_is_none(self):
        import shutil
        self.assertIsNone(gn.measure(shutil.which("yes"), b"", 0.2))


if __name__ == "__main__":
    unittest.main()
