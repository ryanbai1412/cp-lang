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


if __name__ == "__main__":
    unittest.main()
