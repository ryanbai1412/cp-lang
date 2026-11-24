import importlib.util
import os
import tempfile
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

    def test_a_failing_side_shows_its_stderr(self):
        interp = df.Out((3, b""))
        interp.err = b"error[R0001]: internal: lost frame\n  at 4:2\n"
        f = self.outcomes(interp, (0, b"4\n"))
        self.assertIn("interp exit 3, c++ exit 0, interp stderr b'error[R0001]: internal: lost frame | at 4:2'", df.DETAIL[f])


class Interactive(unittest.TestCase):
    """A package with an interactor.py: both sides judged by it, never fed the case."""

    def setUp(self):
        root = tempfile.mkdtemp()
        self.package = os.path.join(root, "corpus", "problems", "p")
        os.makedirs(self.package)
        for name in ("port.cp", "interactor.py", "tests.toml"):
            open(os.path.join(self.package, name), "w").close()
        self.cases = os.path.join(root, "target", "corpus-tests", "p")
        for attr in ("ROOT", "native", "run", "judged", "make_cases"):
            self.addCleanup(setattr, df, attr, getattr(df, attr))
        df.ROOT = root
        df.native = lambda pid, port, cpl: "exe"
        df.run = lambda cmd, inp: self.fail("an interactive case was fed to a side as stdin")
        df.DETAIL.clear()
        self.calls = []
        self.made = []

    def make(self, pid, cpl):
        self.made.append(pid)
        os.makedirs(self.cases, exist_ok=True)
        for name in ("01", "02"):
            open(os.path.join(self.cases, f"{name}.in"), "w").close()

    def verdicts(self, port, cxx):
        def judged(cpl, program, *args):
            self.calls.append((program, args))
            ruled = cxx if program == "exe.cpp" else port
            return [{"name": n, "verdict": v, "why": None} for n, v in zip(("01", "02"), ruled)]
        df.judged = judged

    def test_missing_cases_are_made_then_both_sides_judged_once(self):
        df.make_cases = self.make
        self.verdicts(["ok", "ok"], ["ok", "ok"])
        self.assertEqual(df.check("p", "cpl"), [])
        self.assertEqual(self.made, ["p"])
        port, cxx = self.calls
        self.assertTrue(port[0].endswith("port.cp"))
        self.assertEqual(cxx, ("exe.cpp", ("--tests", self.cases, "--interactor", os.path.join(self.package, "interactor.py"))))

    def test_verdicts_that_disagree_are_a_differ(self):
        self.make("p", "cpl")
        self.verdicts(["ok", "ok"], ["ok", "WA"])
        f = df.check("p", "cpl")
        self.assertEqual(f, ["differ p target/corpus-tests/p/02.in"])
        self.assertIn("interp ok", df.DETAIL[f[0]])


if __name__ == "__main__":
    unittest.main()
