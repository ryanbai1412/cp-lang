import importlib.util
import pathlib
import unittest

spec = importlib.util.spec_from_file_location("flaky", pathlib.Path(__file__).with_name("flaky.py"))
fl = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fl)

# Verbatim from cargo-nextest 0.9.152 with --retries 1.
LOG = """\
        PASS [   0.011s] (1/2) flaky tests::it_works
  TRY 2 PASS [   0.006s] (2/2) flaky sometimes
────────────
     Summary [   0.018s] 2 tests run: 2 passed (1 flaky), 0 skipped
   FLAKY 2/2 [   0.006s] (2/2) flaky sometimes
"""

# Verbatim from cargo-nextest 0.9.152 with --retries 1 --flaky-result fail.
FAILED = """\
  TRY 2 PASS [   0.004s] (2/2) flk flips
           - test configured to fail if flaky
────────────
     Summary [   0.008s] 2 tests run: 1 passed, 1 failed, 0 skipped
 FLKY-FL 2/2 [   0.004s] (2/2) flk flips
error: test run failed
"""


class Flaky(unittest.TestCase):
    def test_names_each_test_that_passed_only_on_retry(self):
        self.assertEqual(fl.flaky(LOG), ["flaky sometimes"])

    def test_names_a_flake_that_failed_the_run(self):
        self.assertEqual(fl.flaky(FAILED), ["flk flips"])

    def test_a_clean_run_has_none(self):
        self.assertEqual(fl.flaky(LOG.replace("FLAKY", "PASS")), [])


if __name__ == "__main__":
    unittest.main()
