import importlib.util
import unittest
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location("differential", Path(__file__).resolve().parent.parent / "corpus" / "differential.py")
df = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(df)


class Compare(unittest.TestCase):
    def setUp(self):
        self.old = df.run
        self.addCleanup(setattr, df, "run", self.old)
        df.SKIPPED.clear()
        df.DETAIL.clear()

    def outcomes(self, interp, *cxx):
        """`run` gives the interpreter's result, then each C++ result in turn."""
        results = iter([interp, *cxx])
        df.run = lambda cmd, inp: next(results)
        return df.compare("p", "port.cp", "exe", f"{df.ROOT}/t/1.in", "cpl")

    def test_agreement_is_no_finding(self):
        self.assertIsNone(self.outcomes((0, b"1\n"), (0, b"1\n")))

    def test_one_cxx_timeout_is_retried_not_a_finding(self):
        self.assertIsNone(self.outcomes((0, b"1\n"), None, (0, b"1\n")))

    def test_two_cxx_timeouts_are_a_finding_that_says_so(self):
        f = self.outcomes((0, b"1\n"), None, None)
        self.assertEqual(f, "differ p t/1.in")
        self.assertIn("c++ timed out twice", df.DETAIL[f])

    def test_an_interp_timeout_is_skipped(self):
        self.assertIsNone(self.outcomes(None))
        self.assertEqual(df.SKIPPED, ["p 1.in"])

    def test_a_differ_names_both_exit_codes_and_the_first_differing_line(self):
        f = self.outcomes((0, b"a\nb\nc\n"), (0, b"a\nB\nc\n"))
        self.assertEqual(df.DETAIL[f], "interp exit 0, c++ exit 0; stdout line 2: interp b'b', c++ b'B'")

    def test_a_crash_shows_its_exit_code(self):
        f = self.outcomes((0, b"a\n"), (-11, b""))
        self.assertTrue(df.DETAIL[f].startswith("interp exit 0, c++ exit -11"))

    def test_a_failing_side_names_its_first_stderr_line(self):
        interp = df.Out((3, b""))
        interp.err = b"error[R0001]: internal: lost frame\n  at 4:2\n"
        f = self.outcomes(interp, (0, b"4\n"))
        self.assertIn("interp exit 3, c++ exit 0, interp stderr b'error[R0001]: internal: lost frame'", df.DETAIL[f])


if __name__ == "__main__":
    unittest.main()
