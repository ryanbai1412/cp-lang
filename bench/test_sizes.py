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
        for path in sizes.oversize() | sizes.unbuilt():
            self.assertTrue(os.path.exists(os.path.join(sizes.ROOT, path)), path)


class BuildVerdict(unittest.TestCase):
    def test_an_unlisted_bench_program_that_does_not_build_fails(self):
        messages, failed = sizes.build_verdict([("bench/a/b.cp", "error")], [], set())
        self.assertTrue(failed)
        self.assertIn("unbuilt.txt", messages[0])

    def test_a_listed_bench_program_that_does_not_build_passes(self):
        _, failed = sizes.build_verdict([("bench/a/b.cp", "error")], [], {"bench/a/b.cp"})
        self.assertFalse(failed)

    def test_a_corpus_program_that_does_not_build_is_left_to_its_suites(self):
        _, failed = sizes.build_verdict([("corpus/problems/x/port.cp", "error")], [], set())
        self.assertFalse(failed)

    def test_a_listed_bench_program_that_builds_must_come_off_the_list(self):
        messages, failed = sizes.build_verdict([], ["bench/a/b.cp"], {"bench/a/b.cp"})
        self.assertTrue(failed)
        self.assertIn("take it off", messages[0])


if __name__ == "__main__":
    unittest.main()
