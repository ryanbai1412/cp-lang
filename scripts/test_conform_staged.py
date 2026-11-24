import sys
import unittest
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


if __name__ == "__main__":
    unittest.main()
