import importlib.util
import os
import unittest

spec = importlib.util.spec_from_file_location("rss", os.path.join(os.path.dirname(__file__), "rss.py"))
rss = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rss)


class Verdict(unittest.TestCase):
    def test_an_unlisted_program_over_the_bar_fails(self):
        messages, failed = rss.verdict({("a", "idiom"): 1.8}, {})
        self.assertTrue(failed)
        self.assertIn("not in", messages[0])

    def test_a_listed_program_within_its_ratio_passes(self):
        _, failed = rss.verdict({("a", "idiom"): 1.9}, {("a", "idiom"): 1.8})
        self.assertFalse(failed)

    def test_a_listed_program_that_grows_fails(self):
        messages, failed = rss.verdict({("a", "idiom"): 2.5}, {("a", "idiom"): 1.8})
        self.assertTrue(failed)
        self.assertIn("grew", messages[0])

    def test_a_listed_program_within_the_bar_must_come_off(self):
        messages, failed = rss.verdict({("a", "tuned"): 1.05}, {("a", "tuned"): 1.3})
        self.assertTrue(failed)
        self.assertIn("take it off", messages[0])

    def test_tuned_is_the_better_of_idiomatic_and_tuned(self):
        got = rss.ratios([{"name": "a", "rss_ratio": 1.4, "tuned_rss_ratio": 1.02}, {"name": "b", "rss_ratio": 0.9}])
        self.assertEqual(got[("a", "tuned")], 1.02)
        self.assertEqual(got[("b", "tuned")], 0.9)

    def test_the_list_parses(self):
        rss.listed()


if __name__ == "__main__":
    unittest.main()
