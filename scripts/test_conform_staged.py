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
    def test_a_failing_case_reports_its_message_and_detail(self):
        got = conform_staged.new_failures("a/wrong.cp", OUT, {})
        self.assertEqual(len(got), 2)
        self.assertIn("write `at 2`", got[0])

    def test_a_case_the_scoreboard_lists_as_failing_passes(self):
        self.assertEqual(conform_staged.new_failures("a/wrong.cp", OUT, {"a/wrong.cp": "fail"}), [])

    def test_only_the_named_case_counts(self):
        self.assertEqual(conform_staged.new_failures("b/ok.cp", OUT, {}), [])


if __name__ == "__main__":
    unittest.main()
