import importlib.util
import os
import unittest

spec = importlib.util.spec_from_file_location("sizes", os.path.join(os.path.dirname(__file__), "sizes.py"))
sizes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sizes)


class Verdict(unittest.TestCase):
    def test_a_listed_program_over_the_bar_passes(self):
        messages, failed = sizes.verdict([(40000, "a.cp")], {"a.cp"})
        self.assertFalse(failed)
        self.assertIn("listed", messages[0])

    def test_an_unlisted_program_over_the_bar_fails(self):
        _, failed = sizes.verdict([(40000, "a.cp")], set())
        self.assertTrue(failed)

    def test_a_listed_program_that_fits_must_come_off_the_list(self):
        messages, failed = sizes.verdict([], {"a.cp"})
        self.assertTrue(failed)
        self.assertIn("take it off", messages[0])

    def test_every_listed_program_exists(self):
        for path in sizes.oversize():
            self.assertTrue(os.path.exists(os.path.join(sizes.ROOT, path)), path)


if __name__ == "__main__":
    unittest.main()
