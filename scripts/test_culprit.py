import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from culprit import failing_tests, nextest_cmd, summary  # noqa: E402

LOG = """\
2026-11-23T17:15:19.6Z        FAIL [   0.878s] (28/34) cpl-cli::runtime_errors wide_range_overflow_traps
2026-11-23T17:15:19.7Z        FAIL [   0.878s] (28/34) cpl-cli::runtime_errors wide_range_overflow_traps
2026-11-23T17:15:20.0Z     TIMEOUT [ 120.001s] cpl-lsp tests::parity::the_editor_shows_the_errors
2026-11-23T17:15:21.0Z        PASS [   0.100s] (29/34) cpl-cli::diagnostics golden_diagnostics
     Summary [  12.119s] 34 tests run: 33 passed, 1 failed, 468 skipped
"""


class CulpritTest(unittest.TestCase):
    def test_reads_each_failing_test_once(self):
        self.assertEqual(failing_tests(LOG), [
            ("cpl-cli::runtime_errors", "wide_range_overflow_traps"),
            ("cpl-lsp", "tests::parity::the_editor_shows_the_errors")])

    def test_runs_only_the_named_tests(self):
        cmd = nextest_cmd([("cpl-cli::runtime_errors", "a"), ("cpl-lsp", "b::c")])
        self.assertIn("(binary_id(cpl-cli::runtime_errors) & test(=a)) | (binary_id(cpl-lsp) & test(=b::c))", cmd)
        self.assertEqual(cmd[-4:], ["-p", "cpl-cli", "-p", "cpl-lsp"])

    def test_summary_names_the_culprit_and_its_revert(self):
        text = summary({"good": "a" * 40, "bad": "b" * 40, "flaky": ["x y"],
                        "already_failing": [], "unbisected": [],
                        "culprits": [{"sha": "c" * 40, "subject": "check: s", "domain": "compiler",
                                      "tests": ["t u"], "revert": {"clean": True, "passes": True,
                                                                   "branch": "culprit/revert-ccccccccc"}}]})
        self.assertIn("ccccccccc (compiler) check: s: 1 test(s), revert passes, on culprit/revert-ccccccccc", text)
        self.assertIn("flaky: x y", text)


if __name__ == "__main__":
    unittest.main()
