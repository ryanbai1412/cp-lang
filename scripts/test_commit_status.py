import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from commit_status import culprit_statuses, statuses  # noqa: E402

A, B, C = "a" * 40, "b" * 40, "c" * 40


class CommitStatusTest(unittest.TestCase):
    def test_green_run_covers_every_commit(self):
        self.assertEqual(statuses([C, B, A], 7, "success", None), [
            (C, "success", "passed, covered by run 7"),
            (B, "success", "passed, covered by run 7"),
            (A, "success", "passed, covered by run 7")])

    def test_red_run_fails_the_tip_and_leaves_the_rest_untested(self):
        self.assertEqual(statuses([C, B], 8, "failure", None), [
            (C, "failure", "run 8 failed here"),
            (B, "pending", "untested: run 8 failed at ccccccccc")])

    def test_empty_range_posts_nothing(self):
        self.assertEqual(statuses([], 9, "success", None), [])

    def test_culprit_gets_a_failure_naming_its_tests(self):
        report = {"culprits": [{"sha": B, "tests": ["cpl-cli::judges every_port"]}]}
        self.assertEqual(culprit_statuses(report, 8),
                         [(B, "failure", "culprit of run 8: every_port")])


if __name__ == "__main__":
    unittest.main()
