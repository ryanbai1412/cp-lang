import importlib.util
import os
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("sizes", os.path.join(os.path.dirname(__file__), "sizes.py"))
sizes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sizes)


class Verdict(unittest.TestCase):
    def test_a_program_over_the_bar_fails(self):
        messages, failed = sizes.verdict([(40000, "a.cp")])
        self.assertTrue(failed)
        self.assertIn("a.cp is 40000 bytes", messages[0])

    def test_nothing_over_the_bar_passes(self):
        self.assertEqual(sizes.verdict([]), ([], False))

    def test_there_is_no_oversize_list(self):
        self.assertFalse(os.path.exists(os.path.join(sizes.ROOT, "bench", "oversize.txt")))


class Stale(unittest.TestCase):
    def test_a_binary_older_than_a_source_is_stale(self):
        with tempfile.NamedTemporaryFile() as f:
            os.utime(f.name, (0, 0))
            self.assertIsNotNone(sizes.stale(f.name))

    def test_a_missing_binary_is_stale(self):
        self.assertIn("missing", sizes.stale("/nonexistent/cpl"))

    def test_a_binary_newer_than_every_source_is_current(self):
        with tempfile.NamedTemporaryFile() as f:
            os.utime(f.name, (4e9, 4e9))
            self.assertIsNone(sizes.stale(f.name))


class Programs(unittest.TestCase):
    def test_every_corpus_port_spec_and_dream_is_walked(self):
        walked = set(sizes.programs())
        for p in sizes.problems():
            for f in (p.port, p.spec, p.dream):
                if f:
                    self.assertIn(os.path.relpath(f, sizes.ROOT), walked)


class BuildVerdict(unittest.TestCase):
    def test_a_bench_program_that_does_not_build_fails(self):
        messages, failed = sizes.build_verdict([("bench/a/b.cp", "error")])
        self.assertTrue(failed)
        self.assertIn("bench/a/b.cp does not build", messages[0])

    def test_a_corpus_program_that_does_not_build_is_left_to_its_suites(self):
        _, failed = sizes.build_verdict([("corpus/problems/x/port.cp", "error")])
        self.assertFalse(failed)

    def test_there_is_no_unbuilt_list(self):
        self.assertFalse(os.path.exists(os.path.join(sizes.ROOT, "bench", "unbuilt.txt")))


if __name__ == "__main__":
    unittest.main()
