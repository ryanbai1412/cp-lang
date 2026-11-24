import importlib.util
import os
import unittest

spec = importlib.util.spec_from_file_location("startup", os.path.join(os.path.dirname(__file__), "startup", "run.py"))
startup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(startup)


class Verdict(unittest.TestCase):
    def test_over_the_ratio_fails(self):
        messages, failed = startup.verdict(1030, 1000, True)
        self.assertTrue(failed)
        self.assertIn("tiny.cp takes 1030 us", messages[0])

    def test_other_output_fails(self):
        self.assertTrue(startup.verdict(900, 1000, False)[1])

    def test_within_the_ratio_passes(self):
        self.assertEqual(startup.verdict(1020, 1000, True), ([], False))


if __name__ == "__main__":
    unittest.main()
