"""Tests for scripts/test-times.py."""

import importlib.util
import json
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("test_times", Path(__file__).parent / "test-times.py")
tt = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tt)

JUNIT = """<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="nextest-run" tests="3">
  <testsuite name="cpl-cli::judges" tests="2">
    <testcase name="fast" classname="cpl-cli::judges" time="1.5"/>
    <testcase name="slow" classname="cpl-cli::judges" time="30.04"><failure/></testcase>
  </testsuite>
  <testsuite name="cpl-lsp" tests="1">
    <testcase name="tests::slower" classname="cpl-lsp" time="61.25"/>
  </testsuite>
</testsuites>
"""


class TestTimesTest(unittest.TestCase):
    def test_keeps_each_test_over_the_limit_slowest_first_failed_or_not(self) -> None:
        self.assertEqual(list(tt.slow_tests(JUNIT).items()),
                         [("cpl-lsp tests::slower", 61.2), ("cpl-cli::judges slow", 30.0)])

    def test_the_record_names_the_command_and_limit(self) -> None:
        rec = json.loads(tt.record({"cpl-cli::judges slow": 30.0}))
        self.assertEqual(rec["limit_s"], 30.0)
        self.assertIn("-p cpl-stress", rec["command"])
        self.assertEqual(rec["over_limit"], {"cpl-cli::judges slow": 30.0})

    def test_merge_re_times_only_the_binaries_the_report_ran(self) -> None:
        old = {"cpl-cli::judges gone": 90.0, "cpl-cli::judges slow": 50.0, "cpl-cli::budget all": 140.0}
        self.assertEqual(list(tt.merged(old, JUNIT).items()),
                         [("cpl-cli::budget all", 140.0), ("cpl-lsp tests::slower", 61.2),
                          ("cpl-cli::judges slow", 30.0)])


if __name__ == "__main__":
    unittest.main()
