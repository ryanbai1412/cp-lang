import importlib.util
import json
import os
import shutil
import tempfile
import unittest
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location("corpus", Path(__file__).resolve().parent / "corpus.py")
assert _SPEC and _SPEC.loader
corpus = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(corpus)

GEN = "import sys\nprint(1)\nprint(sys.argv[1])\n"
ECHO_CPP = "#include <cstdio>\nint main(){int t,x;scanf(\"%d %d\",&t,&x);printf(\"%d\\n\",x);}\n"


class Corpus(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        for name, value in (("PROBLEMS", self.root / "problems"), ("SETS", self.root / "sets")):
            self.addCleanup(setattr, corpus, name, getattr(corpus, name))
            setattr(corpus, name, str(value))
        (self.root / "sets").mkdir()
        (self.root / "sets" / "small.txt").write_text("p2\n# comment\n")

    def package(self, pid, **files):
        d = self.root / "problems" / pid
        d.mkdir(parents=True)
        for name, text in files.items():
            (d / name.replace("_", ".")).write_text(text)
        return d


class ProblemsTest(Corpus):
    def test_lists_every_package_sorted_with_its_files(self):
        self.package("p2", port_cp="", gen_py=GEN)
        self.package("p1", dream_cp="", orig_cpp="", tests_toml="")
        (self.root / "problems" / "README.md").write_text("")
        got = list(corpus.problems())
        self.assertEqual([p.id for p in got], ["p1", "p2"])
        p1, p2 = got
        self.assertEqual((p1.dream.name, p1.orig.name, p1.tests.name), ("dream.cp", "orig.cpp", "tests.toml"))
        self.assertIsNone(p1.port)
        self.assertEqual((p2.port.name, p2.gen.name), ("port.cp", "gen.py"))
        self.assertIsNone(p2.checker)
        self.assertIsNone(p2.interactor)

    def test_filters_by_set_and_by_id(self):
        for pid in ("p1", "p2", "p3"):
            self.package(pid)
        self.assertEqual([p.id for p in corpus.problems(sets=["small"])], ["p2"])
        self.assertEqual([p.id for p in corpus.problems(ids=["p3", "p1"])], ["p1", "p3"])
        self.assertEqual([p.id for p in corpus.problems(sets=["small"], ids=["p1"])], [])

    def test_case_inputs_come_from_tests_toml(self):
        d = self.package("p1", gen_py=GEN, tests_toml='[[case]]\ninput = "1 2"\n\n[[case]]\nseed = 7\nsize = 4\n')
        pkg = corpus.Package.at(d)
        self.assertEqual([pkg.case_input(c) for c in pkg.cases()], ["1 2\n", "1\n7\n"])

    def test_reference_falls_back_to_an_accepted_submission(self):
        d = self.package("p1", orig_cpp="")
        self.assertEqual(corpus.Package.at(d).reference(), d / "orig.cpp")
        self.assertIsNone(corpus.Package.at(self.package("p2", meta_json="{}")).reference())


@unittest.skipUnless(shutil.which("g++") and os.path.exists(os.path.join(corpus.ROOT, "target", "release", "cpl")),
                     "needs g++ and target/release/cpl")
class DreamStressTest(Corpus):
    def run_dream(self, dream):
        d = self.package("p1", dream_cp=dream, orig_cpp=ECHO_CPP, gen_py=GEN, meta_json="{}")
        pkg = corpus.Package.at(d)
        got = corpus.dream_stress(pkg, os.path.join(corpus.ROOT, "target", "release", "cpl"))
        corpus.record(pkg, "dream_stress", got)
        self.assertEqual(json.loads((d / "meta.json").read_text())["dream_stress"], got)
        self.assertEqual(got["sha"], corpus.blob_sha(d / "dream.cp"))
        return got

    def test_an_agreeing_dream_is_ok(self):
        got = self.run_dream("let _t = read<Int>()\nlet x = read<Int>()\nprint(x)\n")
        self.assertEqual((got["ok"], got["cases"]), (True, 2 * len(corpus.STRESS_SIZES)))

    def test_a_disagreeing_dream_is_not(self):
        got = self.run_dream("let _t = read<Int>()\nlet x = read<Int>()\nprint(x + 1)\n")
        self.assertFalse(got["ok"])


if __name__ == "__main__":
    unittest.main()
