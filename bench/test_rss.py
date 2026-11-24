import importlib.util
import os
import unittest

spec = importlib.util.spec_from_file_location("rss", os.path.join(os.path.dirname(__file__), "rss", "run.py"))
rss = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rss)


class Verdict(unittest.TestCase):
    def test_a_peak_over_the_bound_fails(self):
        messages, failed = rss.verdict([("read_ints", 1536 + rss.BOUND_KB + 1, 1536, True)])
        self.assertTrue(failed)
        self.assertIn("read_ints peaks at", messages[0])

    def test_other_output_fails(self):
        self.assertTrue(rss.verdict([("floor", 1536, 1536, False)])[1])

    def test_within_the_bound_passes(self):
        self.assertEqual(rss.verdict([("floor", 3072, 1536, True)]), ([], False))

    def test_every_case_has_its_program_and_hand_cpp(self):
        for case in rss.CASES:
            for f in (case + ".cp", case + "_hand.cpp"):
                self.assertTrue(os.path.exists(os.path.join(rss.HERE, f)), f)


if __name__ == "__main__":
    unittest.main()
