import importlib.util
import os
import tempfile
import unittest
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location("corpus_tests", Path(__file__).resolve().parent / "corpus-tests.py")
assert _SPEC and _SPEC.loader
ct = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(ct)

GEN = "import sys\nseed, size = sys.argv[1:3] if sys.argv[1] != '--max' else ('0', '99')\nprint(seed, size)\n"


class SeedListTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.old = ct.PROBLEMS
        ct.PROBLEMS = self.tmp.name
        self.addCleanup(setattr, ct, "PROBLEMS", self.old)
        self.dir = Path(self.tmp.name, "p1")
        self.dir.mkdir()
        (self.dir / "gen.py").write_text(GEN)

    def write(self, toml):
        (self.dir / "tests.toml").write_text(toml)

    def test_a_port_with_a_seed_list_has_tests_without_a_spec(self):
        self.assertEqual(ct.problems([]), [])
        (self.dir / "port.cp").write_text("")
        self.assertEqual(ct.problems([]), [])
        self.write("[[case]]\nseed = 1\nsize = 4\n")
        self.assertEqual(ct.problems([]), ["p1"])

    def test_no_list_means_none(self):
        self.assertIsNone(ct.seed_list("p1"))

    def test_seed_size_args_and_inline_cases(self):
        self.write('[[case]]\nseed = 7\nsize = 20\n\n[[case]]\nargs = ["--max"]\n\n[[case]]\ninput = "3\\n1 2 3"\n')
        cases = ct.seed_list("p1")
        self.assertEqual(
            [ct.case_input("p1", c) for c in cases],
            ["7 20\n", "0 99\n", "3\n1 2 3\n"],
        )

    def test_list_replaces_the_default_cases(self):
        self.write("[[case]]\nseed = 1\nsize = 4\n")
        self.assertEqual(ct.inputs("p1", {}, 10, self.tmp.name), ["1 4\n"])

    def test_stamp_follows_the_list_and_generator(self):
        self.write("[[case]]\nseed = 1\nsize = 4\n")
        first = ct.stamp("p1", None)
        self.assertEqual(first, ct.stamp("p1", None))
        self.write("[[case]]\nseed = 2\nsize = 4\n")
        self.assertNotEqual(first, ct.stamp("p1", None))
        second = ct.stamp("p1", None)
        (self.dir / "gen.py").write_text(GEN + "# changed\n")
        self.assertNotEqual(second, ct.stamp("p1", None))

    def test_seed_keeps_small_inputs_inline_and_sizes_large_ones(self):
        tests = self.dir / "tests"
        tests.mkdir()
        (tests / "1.in").write_text("tiny\n")
        (tests / "2.in").write_text("x" * (ct.INLINE + 1))
        old = ct.case_input
        ct.case_input = lambda p, c, timeout=60: "y" * (c["size"] * 10)
        self.addCleanup(setattr, ct, "case_input", old)
        ct.seed(type("A", (), {"ids": ["p1"]}))
        cases = ct.tomllib.loads((self.dir / "tests.toml").read_text())["case"]
        self.assertEqual(cases[0], {"input": "tiny\n"})
        self.assertEqual(cases[1]["seed"], 2)
        self.assertLessEqual(cases[1]["size"] * 10, ct.INLINE + 1)
        self.assertGreater(cases[1]["size"] * 20, ct.INLINE + 1)


if __name__ == "__main__":
    unittest.main()
