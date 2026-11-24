import importlib.util
import json
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

    def test_the_reference_is_the_first_accepted_cpp_not_java(self):
        java, cpp = self.dir / "1.java", self.dir / "2.cpp"
        java.write_text("")
        cpp.write_text("")
        subs = [{"file": str(java), "result": "AC"}, {"file": str(cpp), "result": "AC"}]
        (self.dir / "meta.json").write_text(json.dumps({"submissions": subs}))
        self.assertEqual(ct.accepted_cpp("p1"), str(cpp))

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


class InteractiveTest(unittest.TestCase):
    def test_an_interactive_package_writes_hidden_tests_and_judges_the_cpp_with_cpl_test(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        root = Path(tmp.name)
        problems, out = root / "problems", root / "out"
        p = problems / "p1"
        p.mkdir(parents=True)
        (p / "gen.py").write_text(GEN)
        (p / "port.cp").write_text("")
        (p / "interactor.py").write_text("")
        (p / "tests.toml").write_text("[[case]]\nseed = 1\nsize = 4\n\n[[case]]\nseed = 2\nsize = 4\n")
        (p / "meta.json").write_text('{"submissions": [{"result": "AC", "file": "%s"}]}' % (root / "ac.cpp"))
        (root / "ac.cpp").write_text("")
        calls = root / "calls"
        cpl = root / "cpl"
        cpl.write_text("#!/bin/sh\necho \"$@\" >> %s\n"
                       "echo '{\"cases\": [{\"name\": \"01\", \"verdict\": \"ok\"}, {\"name\": \"02\", \"verdict\": \"ok\"}]}'\n" % calls)
        cpl.chmod(0o755)
        for name, value in (("PROBLEMS", str(problems)), ("CPL", str(cpl))):
            self.addCleanup(setattr, ct, name, getattr(ct, name))
            setattr(ct, name, value)
        old = ct.binary
        ct.binary = lambda src: self.fail("an interactive package's C++ never runs alone")
        self.addCleanup(setattr, ct, "binary", old)
        ct.make(type("A", (), {"ids": ["p1"], "seeded": False, "out": str(out), "cases": 8}))
        self.assertEqual(sorted(f.name for f in (out / "p1").iterdir()), [".stamp", "01.in", "02.in"])
        argv = calls.read_text().split()
        self.assertEqual(argv[:2], ["test", str(root / "ac.cpp")])
        self.assertEqual(argv[argv.index("--interactor") + 1], str(p / "interactor.py"))
        self.assertEqual(len(calls.read_text().splitlines()), 1)


if __name__ == "__main__":
    unittest.main()
