import subprocess
import sys
import unittest
from unittest import mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import conform_staged  # noqa: E402

OUT = """conformance     1/2    pass (1 fail)
  a/wrong.cp: fail: wanted E0302 at 4 (it is on file line 3: write `at 2`)
  --> spec/conformance/a/wrong.cp:3:14
  ab/wrong.cp: fail: wanted E0001
cpl-conform: finished in 0.02s
"""


class NewFailures(unittest.TestCase):
    def test_a_new_failing_case_reports_its_message_and_detail(self):
        got = conform_staged.new_failures("a/wrong.cp", OUT, set())
        self.assertEqual(len(got), 2)
        self.assertIn("write `at 2`", got[0])

    def test_a_case_the_board_records_failing_passes(self):
        self.assertEqual(conform_staged.new_failures("a/wrong.cp", OUT, {"a/wrong.cp"}), [])

    def test_board_failing_reads_the_board(self):
        board = '{"suites": {"conformance": {"cases": {"a/wrong.cp": "fail", "b/ok.cp": "pass"}}}}'
        self.assertEqual(conform_staged.board_failing(board), {"a/wrong.cp"})
        self.assertEqual(conform_staged.board_failing(None), set())

    def test_only_the_named_case_counts(self):
        self.assertEqual(conform_staged.new_failures("b/ok.cp", OUT, set()), [])


class Flake(unittest.TestCase):
    def test_a_verdict_that_changes_between_runs_is_a_flake(self):
        passed = "conformance     1/1    pass (1 pass)\n"
        got = conform_staged.flake("a/wrong.cp", OUT, passed)
        self.assertEqual(got, ["  a/wrong.cp: fail: flaky, fail then pass on the same build"])

    def test_the_same_verdict_twice_is_no_flake(self):
        self.assertEqual(conform_staged.flake("a/wrong.cp", OUT, OUT), [])
        self.assertEqual(conform_staged.flake("b/ok.cp", OUT, OUT), [])


CASE = "std-algebra/reversing-a-fold-with-no-rev-traps.cp"
RED = f"conformance     0/1    fail (1 fail)\n  {CASE}: fail: wanted a trap\n"


class AtBase(unittest.TestCase):
    """f33d10f7 was refused for a case red on main: land and the hook (both pass --base)
    refuse only a case that passes at the base."""

    def _main(self, at_base, base=("--base", "HEAD")):
        run = lambda argv, **_: subprocess.CompletedProcess(argv, 0, RED, "")
        log = []
        with mock.patch.object(conform_staged, "base_verdicts", lambda cases, rev, run: {c: at_base for c in cases}), \
                mock.patch.object(conform_staged, "owner", lambda c, rev: "stdlib"), \
                mock.patch.object(conform_staged, "head_board", lambda: None), \
                mock.patch("builtins.open", mock.mock_open()) as opened, \
                mock.patch("sys.stderr") as err:
            code = conform_staged.main([*base, "conform", conform_staged.PREFIX + CASE], run=run)
            log = "".join(c.args[0] for c in err.write.call_args_list)
        return code, log, opened

    def test_a_case_red_at_the_base_is_main_s(self):
        code, err, opened = self._main("fail")
        self.assertEqual(code, 0)
        self.assertIn(f"pre-existing red, owner stdlib: spec/conformance/{CASE} fails at the base", err)
        opened().write.assert_called_once()

    def test_a_case_passing_unbuilt_or_absent_at_the_base_refuses(self):
        for verdict in ("pass", "unbuilt", "absent"):
            code, err, _ = self._main(verdict)
            self.assertEqual(code, 1, verdict)
            self.assertIn(f"NEW red: spec/conformance/{CASE} is {verdict} at the base", err)

    def test_without_a_base_a_new_failure_refuses(self):
        self.assertEqual(self._main("fail", base=())[0], 1)



class FullSuite(unittest.TestCase):
    """`--all`: cea2d50e4 changed only compiler crates and prelude std, and a case calling prelude
    `bfs` with no `use` went red on main; the whole suite runs, refusing only a red the base passes."""

    def test_the_prelude_reaches_what_it_uses_transitively(self):
        files = {"std/prelude.cp": "use std.graph.{bfs}\n", "std/graph.cp": "use std.seq.{Seq}\n",
                 "std/seq.cp": "fn f() = 1\n"}

        def read(p):
            if p not in files:
                raise OSError(p)
            return files[p]
        self.assertEqual(conform_staged.prelude_reach(read), set(files))

    def test_a_compiler_crate_or_prelude_std_needs_the_suite(self):
        reach = {"std/prelude.cp", "std/graph.cp"}
        self.assertTrue(conform_staged.needs_all(["crates/cpl-check/src/check/assoc.rs"], reach))
        self.assertTrue(conform_staged.needs_all(["std/graph.cp"], reach))
        self.assertFalse(conform_staged.needs_all(["scripts/land.sh", "std/timetree.cp", "crates/cpl-lsp/src/a.rs"],
                                                  reach))

    def test_all_failing_names_each_failing_case(self):
        self.assertEqual(sorted(conform_staged.all_failing(OUT)), ["a/wrong.cp", "ab/wrong.cp"])

    def _run_all(self, at_base, alone, red=RED, base_now=""):
        def run(argv, **_):
            out = red if argv[-1] == "--verbose" or alone == "fail" else "conformance 1/1 pass\n"
            return subprocess.CompletedProcess(argv, 0, out, "")
        with mock.patch.object(conform_staged, "base_verdicts", lambda cases, rev, run: {c: at_base for c in cases}), \
                mock.patch.object(conform_staged, "base_alone", lambda c, rev, run: base_now), \
                mock.patch.object(conform_staged, "owner", lambda c, rev: "compiler"), \
                mock.patch("builtins.open", mock.mock_open()), \
                mock.patch("sys.stderr") as err, mock.patch("sys.stdout"):
            code = conform_staged.run_all("conform", "HEAD", run=run)
        return code, "".join(c.args[0] for c in err.write.call_args_list)

    def test_a_case_red_at_the_base_too_is_mains(self):
        code, said = self._run_all("fail", "fail")
        self.assertEqual(code, 0)
        self.assertIn("pre-existing red, owner compiler", said)

    def test_a_case_green_at_the_base_and_red_alone_refuses(self):
        code, said = self._run_all("pass", "fail")
        self.assertEqual(code, 1)
        self.assertIn("NEW red", said)

    def test_a_case_that_passes_alone_is_a_race_not_refused(self):
        code, said = self._run_all("pass", "pass")
        self.assertEqual(code, 0)
        self.assertIn("flaky at the tip", said)

    def test_a_timeout_the_base_also_hits_under_the_same_load_is_not_refused(self):
        slow = f"conformance     0/1    fail (1 fail)\n  {CASE}: fail\n    the interpreter timed out after 20s\n"
        code, said = self._run_all("pass", "fail", red=slow, base_now=slow)
        self.assertEqual(code, 0)
        self.assertIn("times out alone here and at the base", said)

    def test_a_timeout_the_base_beats_under_the_same_load_refuses(self):
        slow = f"conformance     0/1    fail (1 fail)\n  {CASE}: fail\n    the interpreter timed out after 20s\n"
        code, said = self._run_all("pass", "fail", red=slow, base_now="conformance 1/1 pass\n")
        self.assertEqual(code, 1)
        self.assertIn("NEW red", said)

if __name__ == "__main__":
    unittest.main()


class Check(unittest.TestCase):
    """`--check`: a corpus program failing `cpl check` refuses only if it checks at the base."""

    def _main(self, at_base):
        run = lambda argv, **_: subprocess.CompletedProcess(argv, 1 if "check" in argv else 0, "", "E0301")
        prog = "corpus/problems/noi09p4/spec.cp"
        with mock.patch.object(conform_staged, "base_checks", lambda ps, rev, run: {p: at_base for p in ps}), \
             mock.patch.object(conform_staged, "owner", lambda c, rev, path=None: "compiler"), \
             mock.patch("builtins.open", mock.mock_open()) as opened, \
             mock.patch("sys.stderr") as err:
            code = conform_staged.main(["--base", "HEAD", "--check", prog], run=run)
        said = "".join(c.args[0] for c in err.write.call_args_list)
        return code, opened, said

    def test_a_program_red_at_the_base_too_is_mains_and_is_logged(self):
        code, opened, said = self._main("fail")
        self.assertEqual(code, 0)
        self.assertIn("pre-existing red, owner compiler", said)
        self.assertIn("check:corpus/problems/noi09p4/spec.cp", "".join(
            c.args[0] for c in opened().write.call_args_list))

    def test_a_program_that_checks_at_the_base_refuses(self):
        for at in ("pass", "unbuilt", "absent"):
            code, _, said = self._main(at)
            self.assertEqual(code, 1, at)
            self.assertIn("NEW red", said)
