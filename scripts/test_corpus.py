import argparse
import contextlib
import io
import importlib.util
import json
import os
import shutil
import subprocess
import tempfile
import types
import unittest
from unittest import mock
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

    def test_accepted_cpp_builds_with_the_judges_gxx(self):
        src = self.root / "judged.cpp"
        src.write_text("#include <bits/extc++.h>\nusing namespace std;\n"
                       "struct S { int v; int get(this S const& s) { return s.v; } };\n"
                       "int main(){int n=3; int a[n] {}; vector<int> r(3); ranges::iota(r, 1);\n"
                       "__gnu_pbds::tree<int, __gnu_pbds::null_type> t; t.insert(r[2]);\n"
                       "printf(\"%d %d %d\\n\", a[2], S{4}.get(), (int)*t.begin());}\n")
        self.addCleanup(setattr, corpus, "CXX_CACHE", corpus.CXX_CACHE)
        corpus.CXX_CACHE = str(self.root / "cxx")
        out = subprocess.run([corpus.binary(src)], capture_output=True, text=True).stdout
        self.assertEqual(out, "0 4 3\n")

    def test_a_gxx_older_than_14_is_not_the_reference(self):
        old = self.root / "g++"
        old.write_text("#!/bin/sh\necho 13.3.0\n")
        old.chmod(0o755)
        with mock.patch("shutil.which", lambda name: str(old) if name == "g++" else None), \
                mock.patch.dict(os.environ, CPL_GXX_FETCH="0"), self.assertRaises(RuntimeError) as e:
            corpus.reference_cxx.__wrapped__()
        self.assertIn("g++ 14", str(e.exception))

    def test_with_no_gxx_14_on_path_the_fetched_one_is_the_reference(self):
        new = self.root / "fetched-g++"
        new.write_text("#!/bin/sh\necho 14.4.0\n")
        new.chmod(0o755)
        fetch = subprocess.CompletedProcess([], 0, stdout=f"{new}\n")
        real_run = subprocess.run
        run = lambda cmd, *a, **k: fetch if cmd[-1].endswith("gxx14.sh") else real_run(cmd, *a, **k)
        with mock.patch("shutil.which", lambda name: None), mock.patch("subprocess.run", run), \
                mock.patch.dict(os.environ, CPL_GXX_FETCH="1"):
            cxx, version = corpus.reference_cxx.__wrapped__()
        self.assertEqual((cxx[0], version), (str(new), "14.4.0"))

    def test_a_checker_judges_agreement_at_max_and_in_stress(self):
        d = self.package("p1", checker_py="import sys\nsys.exit(0 if open(sys.argv[2]).read().strip() in ('1', '2') else 1)\n")
        self.assertTrue(corpus.agrees(corpus.Package.at(d), b"", b"2\n", b"1\n"))
        self.assertFalse(corpus.agrees(corpus.Package.at(d), b"", b"3\n", b"1\n"))
        self.assertTrue(corpus.agrees(corpus.Package.at(self.package("p2")), b"", b"1 2", b"1\n2\n"))

    def test_a_package_named_on_the_command_line_runs_outside_the_run_set(self):
        pkg = corpus.Package.at(self.package("p1"))
        self.assertFalse(corpus.Context("cpl").runs(pkg) and "p1" not in corpus.Context("cpl").run_ids)
        self.assertTrue(corpus.Context("cpl", named=["p1"]).runs(pkg))
        self.assertFalse(corpus.Context("cpl", run=False, named=["p1"]).runs(pkg))

    def test_a_binary_appears_only_once_fully_built(self):
        seen = []
        def cxx(cmd, *a, **k):
            out = cmd[cmd.index("-o") + 1]
            Path(out).write_text("partial")
            seen.append(os.path.exists(os.path.join(corpus.CXX_CACHE, key)))
            return subprocess.CompletedProcess(cmd, 0, "", "")
        src = self.root / "a.cpp"
        src.write_text("int main(){}\n")
        self.addCleanup(setattr, corpus, "CXX_CACHE", corpus.CXX_CACHE)
        corpus.CXX_CACHE = str(self.root / "cxx")
        import hashlib
        key = hashlib.sha1(src.read_bytes() + b"\0gcc 14.4.0").hexdigest()[:16]
        with mock.patch.object(corpus, "reference_cxx", lambda: (["g++"], "14.4.0")), \
                mock.patch("subprocess.run", cxx):
            exe = corpus.binary(src)
        self.assertEqual((seen, os.path.basename(exe)), ([False], key))

    def test_a_reference_with_an_avx2_pragma_before_its_includes_builds(self):
        calls = []
        def cxx(cmd, *a, **k):
            calls.append("-mavx2" in cmd)
            if "-mavx2" not in cmd:
                return subprocess.CompletedProcess(cmd, 1, "", "error: inlining failed: target specific option mismatch")
            Path(cmd[cmd.index("-o") + 1]).write_text("exe")
            return subprocess.CompletedProcess(cmd, 0, "", "")
        src = self.root / "a.cpp"
        src.write_text('#pragma GCC target("avx2")\n#include <bits/stdc++.h>\nint main(){}\n')
        self.addCleanup(setattr, corpus, "CXX_CACHE", corpus.CXX_CACHE)
        corpus.CXX_CACHE = str(self.root / "cxx")
        with mock.patch.object(corpus, "reference_cxx", lambda: (["g++"], "14.4.0")), \
                mock.patch("subprocess.run", cxx):
            self.assertTrue(os.path.exists(corpus.binary(src)))
        self.assertEqual(calls, [False, True])

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

    def check(self, dream, meta_json="{}"):
        d = self.package("p1", dream_cp=dream, orig_cpp=ECHO_CPP, gen_py=GEN, meta_json=meta_json)
        cpl = os.path.join(corpus.ROOT, "target", "release", "cpl")
        args = argparse.Namespace(ids=["p1"], cpl=cpl, check=True)
        with mock.patch.object(corpus, "problems", lambda sets=None, ids=None: iter([corpus.Package.at(d)])):
            return corpus.verify(args), json.loads((d / "meta.json").read_text())

    def test_a_broken_toolchain_is_no_verdict_and_writes_nothing(self):
        good = json.dumps({"dream_stress": {"ok": True, "cases": 8, "sha": "abc"}})
        d = self.package("p1", dream_cp="print(1)\n", orig_cpp=ECHO_CPP, gen_py=GEN, meta_json=good)
        args = argparse.Namespace(ids=["p1"], cpl=os.path.join(corpus.ROOT, "target", "release", "cpl"), check=False)
        broken = mock.Mock(side_effect=RuntimeError("no g++ 14 or newer on PATH"))
        err = io.StringIO()
        with mock.patch.object(corpus, "problems", lambda sets=None, ids=None: iter([corpus.Package.at(d)])), \
                mock.patch.object(corpus, "binary", broken), contextlib.redirect_stderr(err):
            self.assertEqual(corpus.verify(args), 2)
        self.assertEqual((d / "meta.json").read_text(), good)
        self.assertIn("no g++ 14 or newer on PATH", err.getvalue())

    def test_verify_without_the_reference_gxx_stops_naming_it(self):
        with mock.patch.object(corpus, "reference_cxx", mock.Mock(side_effect=RuntimeError("no g++ 14"))), \
                self.assertRaises(SystemExit) as e:
            corpus.toolchain(["verify", "p1"])
        self.assertIn("g++ 14", str(e.exception))

    def test_check_fails_a_disagreeing_dream_and_writes_nothing(self):
        self.assertEqual(self.check("let _t = read<Int>()\nlet x = read<Int>()\nprint(x + 1)\n"), (1, {}))

    def test_check_fails_an_agreeing_dream_its_meta_does_not_record(self):
        self.assertEqual(self.check("let _t = read<Int>()\nlet x = read<Int>()\nprint(x)\n")[0], 1)

    def test_check_passes_an_agreeing_dream_its_meta_records(self):
        dream = "let _t = read<Int>()\nlet x = read<Int>()\nprint(x)\n"
        sha = subprocess.run(["git", "hash-object", "--stdin"], input=dream, capture_output=True, text=True).stdout.strip()
        meta = json.dumps({"dream_stress": {"ok": True, "cases": 8, "sha": sha}})
        self.assertEqual(self.check(dream, meta)[0], 0)


class CompareTest(unittest.TestCase):
    def outcome(self, interp, *cxx):
        """`run` gives the interpreter's result, then each C++ result in turn."""
        results = iter([interp, *cxx])
        return corpus.compare(lambda cmd, data: next(results), ["cpl"], ["exe"], b"")

    def test_agreement_is_no_finding(self):
        self.assertIsNone(self.outcome((0, b"1\n", b""), (0, b"1\n", b"")))

    def test_one_cxx_timeout_is_retried_not_a_finding(self):
        self.assertIsNone(self.outcome((0, b"1\n", b""), None, (0, b"1\n", b"")))

    def test_two_cxx_timeouts_are_a_finding_that_says_so(self):
        self.assertEqual(self.outcome((0, b"1\n", b""), None, None), "c++ timed out twice")

    def test_an_interp_timeout_is_skipped(self):
        self.assertIsNone(self.outcome(None))

    def test_a_differ_names_both_exit_codes_and_the_first_differing_line(self):
        self.assertEqual(self.outcome((0, b"a\nb\n", b""), (0, b"a\nB\n", b"")),
                         "interp exit 0, c++ exit 0; stdout line 2: interp b'b', c++ b'B'")

    def test_a_failing_side_shows_its_stderr(self):
        why = self.outcome((3, b"", b"error[R0001]: internal: lost frame\n  at 4:2\n"), (0, b"4\n", b""))
        self.assertIn("interp exit 3, c++ exit 0, interp stderr b'error[R0001]: internal: lost frame | at 4:2'", why)


class LsTest(Corpus):
    def test_lists_dirs_or_one_file_per_package(self):
        self.package("p1", port_cp="")
        self.package("p2", dream_cp="", gen_py="")
        for argv, want in ((["ls"], ["p1", "p2"]), (["ls", "--file", "port.cp"], ["p1/port.cp"]),
                           (["ls", "--set", "small"], ["p2"]),
                           (["ls", "--file", "*.cp", "--file", "gen.py"], ["p1/port.cp", "p2/dream.cp", "p2/gen.py"])):
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                self.assertEqual(corpus.cli(argv), 0)
            got = [os.path.relpath(os.path.join(corpus.ROOT, l), self.root / "problems") for l in out.getvalue().split()]
            self.assertEqual(got, want)


class CheckTestsTest(Corpus):
    """tests mode runs the port on the cases tests.toml makes, never on a missing tests/."""

    def run_tests(self, cases):
        d = self.package("p", port_cp="print(1)\n", tests_toml="", checker_py="")
        calls = []

        def cpl_cases(cpl, program, *args):
            calls.append(args)
            return cases
        ctx = mock.Mock(cpl="cpl")
        with mock.patch.object(corpus.Package, "reference", lambda self: "orig.cpp"), \
                mock.patch.object(corpus, "make_cases", mock.Mock()), mock.patch.object(corpus, "cpl_cases", cpl_cases):
            found = corpus.check_tests(corpus.Package.at(d), ctx)
        return found, calls, d

    def test_judges_the_made_cases_with_the_checker_at_the_limits(self):
        found, (args,), d = self.run_tests([{"name": "01", "verdict": "ok"}])
        self.assertEqual(found, [])
        self.assertEqual(args[:4], ("--tests", os.path.join(corpus.OUT, "p"), "--checker", str(d / "checker.py")))
        self.assertIn("--time-limit", args)

    def test_no_cases_is_a_finding(self):
        found, *_ = self.run_tests("cpl-test-no-cases")
        self.assertEqual(found, ["cpl-test-no-cases p -"])


class CommittedInputsTest(Corpus):
    def test_an_interactive_package_seeds_from_its_cases_not_its_reply_transcripts(self):
        d = self.package("p", port_cp="", interactor_py="")
        (d / "tests").mkdir()
        for name, text in (("01.case", "data"), ("01.in", "replies"), ("01.ans", "")):
            (d / "tests" / name).write_text(text)
        self.assertEqual(corpus.committed_inputs(corpus.Package.at(d)), [("01.case", "data")])

    def test_a_plain_package_seeds_from_its_inputs(self):
        d = self.package("p", port_cp="")
        (d / "tests").mkdir()
        (d / "tests" / "01.in").write_text("1")
        self.assertEqual(corpus.committed_inputs(corpus.Package.at(d)), [("01.in", "1")])


class InteractedTest(Corpus):
    """An interactive package: both sides judged by interactor.py, never fed the case."""

    def judge(self, port, cxx):
        d = self.package("p", port_cp="print(1)\n", interactor_py="")
        cases = self.root / "corpus-tests"
        cases.mkdir()
        for name in ("01", "02"):
            (cases / f"{name}.in").write_text("")
        calls = []

        def cpl_cases(cpl, program, *args):
            calls.append((str(program), args))
            ruled = cxx if str(program).endswith(".cpp") else port
            return [{"name": n, "verdict": v, "why": None} for n, v in zip(("01", "02"), ruled)]
        build = mock.Mock(return_value=mock.Mock(returncode=0))
        ctx = mock.Mock(cpl="cpl")
        with mock.patch.object(corpus, "generated", lambda pkg: sorted(cases.glob("*.in"))), \
                mock.patch.object(corpus, "cpl_cases", cpl_cases), mock.patch.object(corpus.subprocess, "run", build):
            found = corpus.interacted(corpus.Package.at(d), ctx)
        return found, calls, d, cases

    def test_both_sides_are_judged_once_by_the_interactor(self):
        found, calls, d, cases = self.judge(["ok", "ok"], ["ok", "ok"])
        self.assertEqual(found, [])
        (port, a), (cxx, b) = calls
        self.assertTrue(port.endswith("port.cp") and cxx.endswith("p.cpp"))
        self.assertEqual(a, b)
        self.assertEqual(a, ("--tests", str(cases), "--interactor", str(d / "interactor.py")))

    def test_verdicts_that_disagree_are_a_differ(self):
        found, *_ = self.judge(["ok", "ok"], ["ok", "WA"])
        self.assertEqual(found, ["differ p 02.in: interp ok (None), c++ WA (None)"])


class StaticModesTest(Corpus):
    def ctx(self, stderr):
        cpl = self.root / "cpl"
        cpl.write_text(f"#!/bin/sh\nprintf '{stderr}' >&2\n")
        cpl.chmod(0o755)
        return corpus.Context(str(cpl), run=False)

    def test_a_warning_never_fails_a_port(self):
        pkg = corpus.Package.at(self.package("p1", port_cp=""))
        self.assertEqual(corpus.check_check(pkg, self.ctx("warning[L0608]: `s.first` does work\\n")), [])

    def test_an_error_fails_the_port_it_names(self):
        pkg = corpus.Package.at(self.package("p1", port_cp=""))
        self.assertEqual(corpus.check_check(pkg, self.ctx("error[E0608]: `out` moves here\\n")), ["port-check p1 -"])

    def test_a_failing_port_shows_its_errors(self):
        pkg = corpus.Package.at(self.package("p1", port_cp=""))
        with mock.patch("sys.stderr", new_callable=io.StringIO) as err:
            corpus.check_check(pkg, self.ctx("error[E0608]: `out` moves here\\n  --> port.cp:9:5\\n"))
        self.assertIn("error[E0608]", err.getvalue())
        self.assertIn("port.cp:9:5", err.getvalue())

    def test_the_header_needs_source_problem_and_solution(self):
        pkg = corpus.Package.at(self.package("p1", port_cp="// Source: x\n// Solution: y\nprint(1)\n// Problem: late\n"))
        self.assertEqual(corpus.check_header(pkg, None), ["no-header-problem p1 -"])

    def test_committed_tests_are_a_finding(self):
        d = self.package("p1", port_cp="")
        (d / "tests").mkdir()
        self.assertIn("committed-tests p1 -", corpus.check_package(corpus.Package.at(d), self.ctx("")))

    def test_a_package_with_cases_needs_a_reference(self):
        pkg = corpus.Package.at(self.package("p1", port_cp="", tests_toml='[[case]]\ninput = "1"\n', meta_json="{}"))
        self.assertEqual(corpus.check_package(pkg, self.ctx("")), ["no-reference p1 -"])


class SeedTest(Corpus):
    def setUp(self):
        super().setUp()
        self.dir = self.package("p1", gen_py="import sys\nprint('y' * int(sys.argv[2]) * 10)\n")

    def test_seed_keeps_small_inputs_inline_and_sizes_large_ones(self):
        (self.dir / "tests").mkdir()
        (self.dir / "tests" / "1.in").write_text("tiny\n")
        (self.dir / "tests" / "2.in").write_text("x" * (corpus.INLINE + 1))
        corpus.seed(type("A", (), {"ids": ["p1"]}))
        self.assertFalse((self.dir / "tests").exists())
        cases = corpus.Package.at(self.dir).cases()
        self.assertEqual(cases[0], {"input": "tiny\n"})
        self.assertEqual(cases[1], {"seed": 2, "size": 16})
        self.assertEqual([c["size"] for c in cases[2:]], list(corpus.STRESS_SIZES))

    def test_a_large_case_without_a_generator_is_not_seeded(self):
        d = self.package("p2")
        (d / "tests").mkdir()
        (d / "tests" / "1.in").write_text("x" * (corpus.INLINE + 1))
        self.assertEqual(corpus.seed(type("A", (), {"ids": ["p2"]})), 1)
        self.assertTrue((d / "tests").exists())

    def test_stamp_follows_the_list_and_generator(self):
        (self.dir / "tests.toml").write_text("[[case]]\nseed = 1\nsize = 4\n")
        first = corpus.stamp(corpus.Package.at(self.dir), None)
        (self.dir / "tests.toml").write_text("[[case]]\nseed = 2\nsize = 4\n")
        second = corpus.stamp(corpus.Package.at(self.dir), None)
        self.assertNotEqual(first, second)
        (self.dir / "gen.py").write_text("print(2)\n")
        self.assertNotEqual(second, corpus.stamp(corpus.Package.at(self.dir), None))

    def test_make_writes_inputs_and_reference_answers(self):
        d = self.package("p3", gen_py=GEN, orig_cpp=ECHO_CPP, tests_toml='[[case]]\ninput = "1 5"\n\n[[case]]\nseed = 7\nsize = 1\n')
        corpus.make_cases(corpus.Package.at(d), str(self.root / "out"))
        out = self.root / "out" / "p3"
        self.assertEqual([(out / f"0{i}.ans").read_text() for i in (1, 2)], ["5\n", "7\n"])


class AtMax(unittest.TestCase):
    """`at_max` on scripted runs: the checked C++ first, then three judge runs, then three accepted runs."""

    def findings(self, safe, mine, theirs=(0, b"1\n", 0.05, 10), limits=(2000, 256)):
        results = iter([safe, mine, mine, mine, theirs, theirs, theirs])
        for name, fake in (("measure", lambda exe, data, t: next(results)), ("checked", lambda *a: "safe"),
                           ("limits", lambda port: limits)):
            self.addCleanup(setattr, corpus, name, getattr(corpus, name))
            setattr(corpus, name, fake)
        pkg = types.SimpleNamespace(id="p", port="port.cp", checker=None)
        return corpus.at_max(pkg, "cpl", "exe", "orig", b"")

    def test_a_fast_matching_run_is_no_finding(self):
        self.assertEqual(self.findings((0, b"1\n", 0.1, 10), (0, b"1\n", 0.05, 10)), [])

    def test_a_checked_trap_is_a_finding(self):
        self.assertEqual(self.findings((3, b"", 0.1, 10), (0, b"1\n", 0.05, 10)), ["traps p --max"])

    def test_over_the_time_limit(self):
        self.assertIn("over-limit p --max", self.findings((0, b"1\n", 1, 10), (0, b"1\n", 2.5, 10), (0, b"1\n", 2.4, 10)))

    def test_over_the_memory_limit(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), (0, b"1\n", 0.05, 300)), ["over-limit p --max"])

    def test_a_judge_timeout_is_over_limit(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), None), ["over-limit p --max"])

    def test_a_different_answer_disagrees(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), (0, b"2\n", 0.05, 10)), ["disagrees p --max"])

    def test_slow_against_the_accepted_cpp(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), (0, b"1\n", 0.5, 10), (0, b"1\n", 0.2, 10)), ["slow p --max"])


class Limits(unittest.TestCase):
    def test_the_limits_line_and_the_default(self):
        path = Path(os.environ.get("TMPDIR", "/tmp")) / "test_generators_port.cp"
        path.write_text("// Source: x\n// limits: 3000 ms, 512 MB\nlet n: 1..=5 = read()\n")
        self.addCleanup(path.unlink)
        self.assertEqual(corpus.limits(str(path)), (3000, 512))
        path.write_text("let n: 1..=5 = read()\n")
        self.assertEqual(corpus.limits(str(path)), (2000, 256))


class Measure(unittest.TestCase):
    """`measure` reports the program's own peak memory, however much the gate holds."""

    def test_a_small_program_is_small_while_the_gate_is_big(self):
        import shutil
        big = bytearray(300 << 20)
        for i in range(0, len(big), 4096):
            big[i] = 1
        code, out, _, mb = corpus.measure(shutil.which("cat"), b"hi\n", 20)
        self.assertEqual((code, out), (0, b"hi\n"))
        self.assertLess(mb, 50)
        del big

    def test_a_timeout_is_none(self):
        import shutil
        self.assertIsNone(corpus.measure(shutil.which("yes"), b"", 0.2))


if __name__ == "__main__":
    unittest.main()
