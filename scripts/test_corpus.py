import argparse
import contextlib
import io
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import types
import tomllib
import unittest
from unittest import mock
from pathlib import Path

PKGS = "corpus/problems"
SUBS_DIR = "corpus/submissions"

_SPEC = importlib.util.spec_from_file_location("corpus", Path(__file__).resolve().parent / "corpus.py")
assert _SPEC and _SPEC.loader
corpus = importlib.util.module_from_spec(_SPEC)
sys.modules["corpus"] = corpus  # make's worker processes find its functions by name
_SPEC.loader.exec_module(corpus)

GEN = "import sys\nprint(1)\nprint(sys.argv[1])\n"
REQUIRE_CPL = corpus.require_cpl
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
        # Fake cpls stand in for the build here; StaleCplTest covers the check on the real one.
        fresh = mock.patch.object(corpus, "require_cpl", lambda cpl: None)
        fresh.start()
        self.addCleanup(fresh.stop)

    def package(self, pid, **files):
        d = self.root / "problems" / pid
        d.mkdir(parents=True)
        for name, text in files.items():
            (d / name.replace("_", ".")).write_text(text)
        return d


class ProblemsTest(Corpus):
    def test_a_graded_dream_is_what_the_gates_run_without_a_port(self) -> None:
        dream = "@grader(two_phase)\nfn f(x: Int) -> Int = x\n"
        alone = corpus.Package.at(self.package("g1", dream_cp=dream, grader_cpp="int main(){}\n"))
        self.assertEqual(alone.solution, alone.dir / "dream.cp")
        both = corpus.Package.at(self.package("g2", dream_cp=dream, port_cp="print(1)\n", grader_cpp=""))
        self.assertEqual(both.solution, both.dir / "port.cp")
        ungraded = corpus.Package.at(self.package("g3", dream_cp="print(1)\n", grader_cpp=""))
        self.assertIsNone(ungraded.solution)
        no_grader = corpus.Package.at(self.package("g4", dream_cp=dream))
        self.assertIsNone(no_grader.solution)

    def test_build_keeps_a_recorded_statement(self) -> None:
        st = {"url": "https://example.org/t.pdf", "sha256": "ab" * 32, "samples": 0}
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "meta.json")
            with open(path, "w") as f:
                json.dump({"statement": st, "notes": "n", "stale": 1}, f)
            meta = corpus.merged({"problem": "p", "submissions": []}, path)
        self.assertEqual((meta["statement"], meta["notes"]), (st, "n"))
        self.assertNotIn("stale", meta)

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

    def test_shards_cover_every_package_once_by_id(self):
        pkgs = [types.SimpleNamespace(id=f"p{i}") for i in (5, 1, 4, 2, 3)]
        got = [[p.id for p in corpus.sharded(pkgs, f"{n}/2")] for n in (1, 2)]
        self.assertEqual(got, [["p1", "p3", "p5"], ["p2", "p4"]])
        self.assertIs(corpus.sharded(pkgs, None), pkgs)
        with self.assertRaises(SystemExit):
            corpus.sharded(pkgs, "3/2")

    def test_gen_input_runs_gen_py_and_ends_the_input_with_a_newline(self):
        d = self.package("p1", gen_py=GEN)
        self.assertEqual(corpus.Package.at(d).gen_input([7, 4]), "1\n7\n")

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

    def test_a_gen_that_fails_at_a_small_size_is_a_finding(self):
        gen = "import random, sys\nprint(random.randint(1, int(sys.argv[2]) - 1))\n"
        pkg = corpus.Package.at(self.package("p", gen_py=gen, tests_toml="[[case]]\nseed = 1\nsize = 0\n"))
        self.assertIn("gen-fails p size=1", corpus.check_package(pkg, corpus.Context("cpl", run=True)))

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

    def test_parallel_checks_run_one_binary_while_it_is_built(self):
        # A compiler that writes its output in two steps: run early, a binary is
        # busy (ETXTBSY) or half-written; every caller must still run a whole one.
        cxx = self.root / "slow-cxx"
        cxx.write_text('#!/bin/sh\nout=$2\nprintf "#!/bin/sh\\n" > "$out"\nchmod +x "$out"\n'
                       'sleep 0.2\nprintf "echo whole\\n" >> "$out"\n')
        cxx.chmod(0o755)
        src = self.root / "a.cpp"
        src.write_text("int main(){}\n")
        self.addCleanup(setattr, corpus, "CXX_CACHE", corpus.CXX_CACHE)
        corpus.CXX_CACHE = str(self.root / "cxx")

        def check(_):
            exe = corpus.binary(src)
            return subprocess.run([exe], capture_output=True, text=True).stdout.strip()

        import concurrent.futures
        with mock.patch.object(corpus, "reference_cxx", lambda: ([str(cxx)], "14.4.0")), \
                concurrent.futures.ThreadPoolExecutor(16) as pool:
            got = list(pool.map(check, range(64)))
        self.assertEqual(set(got), {"whole"})

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

    def test_a_file_the_accepted_cpp_writes_stays_out_of_the_working_directory(self):
        writes = ECHO_CPP.replace("int main(){", "int main(){fclose(fopen(\"output.txt\",\"w\"));")
        d = self.package("p1", dream_cp="let _t = read<Int>()\nlet x = read<Int>()\nprint(x)\n",
                         orig_cpp=writes, gen_py=GEN, meta_json="{}")
        cwd = self.root / "cwd"
        cwd.mkdir()
        self.addCleanup(os.chdir, os.getcwd())
        os.chdir(cwd)
        pkg = corpus.Package.at(d)
        self.assertTrue(corpus.dream_stress(pkg, os.path.join(corpus.ROOT, "target", "release", "cpl"))["ok"])
        self.assertEqual(corpus.run_out(corpus.binary(pkg.orig), b"1 5\n"), b"5\n")
        self.assertEqual(list(cwd.iterdir()), [])

    def check(self, dream, meta_json="{}"):
        d = self.package("p1", dream_cp=dream, orig_cpp=ECHO_CPP, gen_py=GEN, meta_json=meta_json)
        cpl = os.path.join(corpus.ROOT, "target", "release", "cpl")
        args = argparse.Namespace(ids=["p1"], cpl=cpl, only="dream")
        with mock.patch.object(corpus, "problems", lambda sets=None, ids=None: iter([corpus.Package.at(d)])):
            return corpus.verify(args), json.loads((d / "meta.json").read_text())

    def test_record_writes_an_agreeing_dream_that_verify_then_passes(self):
        dream = "let _t = read<Int>()\nlet x = read<Int>()\nprint(x)\n"
        d = self.package("p1", dream_cp=dream, orig_cpp=ECHO_CPP, gen_py=GEN, meta_json="{}")
        args = argparse.Namespace(ids=["p1"], set=None, cpl=os.path.join(corpus.ROOT, "target", "release", "cpl"), j=1)
        with mock.patch.object(corpus, "problems", lambda sets=None, ids=None: iter([corpus.Package.at(d)])), \
                contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(corpus.record_dreams(args), 0)
        meta = (d / "meta.json").read_text()
        self.assertEqual(json.loads(meta)["dream_stress"]["sha"], corpus.blob_sha(d / "dream.cp"))
        args = argparse.Namespace(ids=["p1"], cpl=args.cpl, only="dream")
        with mock.patch.object(corpus, "problems", lambda sets=None, ids=None: iter([corpus.Package.at(d)])), \
                contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(corpus.verify(args), 0)
        self.assertEqual((d / "meta.json").read_text(), meta)

    def test_record_leaves_a_disagreeing_dream_unrecorded(self):
        d = self.package("p1", dream_cp="let _t = read<Int>()\nlet x = read<Int>()\nprint(x + 1)\n",
                         orig_cpp=ECHO_CPP, gen_py=GEN, meta_json="{}")
        args = argparse.Namespace(ids=["p1"], set=None, cpl=os.path.join(corpus.ROOT, "target", "release", "cpl"), j=1)
        out = io.StringIO()
        with mock.patch.object(corpus, "problems", lambda sets=None, ids=None: iter([corpus.Package.at(d)])), \
                contextlib.redirect_stdout(out), contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(corpus.record_dreams(args), 1)
        self.assertIn("dream-stress p1 fails-on-", out.getvalue())
        self.assertEqual(json.loads((d / "meta.json").read_text()), {})

    def test_a_broken_toolchain_is_no_verdict_and_writes_nothing(self):
        good = json.dumps({"dream_stress": {"ok": True, "cases": 8, "sha": "abc"}})
        d = self.package("p1", dream_cp="print(1)\n", orig_cpp=ECHO_CPP, gen_py=GEN, meta_json=good)
        args = argparse.Namespace(ids=["p1"], cpl=os.path.join(corpus.ROOT, "target", "release", "cpl"))
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

    def test_verify_fails_a_disagreeing_dream_and_writes_nothing(self):
        self.assertEqual(self.check("let _t = read<Int>()\nlet x = read<Int>()\nprint(x + 1)\n"), (1, {}))

    def test_verify_fails_an_agreeing_dream_its_meta_does_not_record_and_writes_nothing(self):
        self.assertEqual(self.check("let _t = read<Int>()\nlet x = read<Int>()\nprint(x)\n"), (1, {}))

    def test_verify_passes_an_agreeing_dream_its_meta_records(self):
        dream = "let _t = read<Int>()\nlet x = read<Int>()\nprint(x)\n"
        sha = subprocess.run(["git", "hash-object", "--stdin"], input=dream, capture_output=True, text=True).stdout.strip()
        meta = json.dumps({"dream_stress": {"ok": True, "cases": 8, "sha": sha}})
        self.assertEqual(self.check(dream, meta)[0], 0)


class CliTest(unittest.TestCase):
    def test_an_unknown_verify_flag_fails_loudly(self):
        for argv in (["verify", "--check", "p1"], ["verify", "--frozen", "p1"]):
            err = io.StringIO()
            with contextlib.redirect_stderr(err), self.assertRaises(SystemExit) as e:
                corpus.cli(argv)
            self.assertEqual(e.exception.code, 2)
            self.assertIn(f"unrecognized arguments: {argv[1]}", err.getvalue())


class InterpSlowTest(Corpus):
    def check_twice(self, *builds):
        """check_stress on one slow case once per `cpl` build in `builds`; the cpl runs it made."""
        d = self.package("p", port_cp="print(1)\n")
        case = self.root / "01.in"
        case.write_text("1\n")
        cpl = self.root / "cpl"
        runs = []

        def run_result(cmd, data):
            runs.append(cmd[0])
            return None if cmd[0] == str(cpl) else (0, b"1\n", b"")
        corpus.build_id.cache_clear()
        with mock.patch.object(corpus, "INTERP_SLOW", str(self.root / "slow")), \
                mock.patch.object(corpus, "native", lambda pkg, ctx: ("exe", 1)), \
                mock.patch.object(corpus, "generated", lambda pkg: [case]), \
                mock.patch.object(corpus, "run_result", run_result):
            pkg = corpus.Package.at(d)
            for build in builds:
                cpl.write_bytes(build)
                corpus.build_id.cache_clear()
                ctx = corpus.Context(str(cpl), named=["p"])
                self.assertEqual(corpus.check_stress(pkg, ctx), [])
                self.assertEqual(ctx.slow, ["p 01.in"])
        return runs.count(str(cpl))

    def test_a_case_past_the_interpreter_is_reported_and_then_skipped(self):
        self.assertEqual(self.check_twice(b"v1", b"v1"), 1)

    def test_a_new_cpl_build_runs_a_slow_case_again(self):
        self.assertEqual(self.check_twice(b"v1", b"v2"), 2)


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

    def test_an_interp_timeout_is_slow_not_agreement(self):
        self.assertEqual(self.outcome(None), corpus.SLOW)

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

    def test_a_package_without_a_port_still_runs_its_reference_on_every_case(self):
        d = self.package("q", tests_toml="", checker_py="")
        made = mock.Mock()
        ctx = mock.Mock(cpl="cpl")
        with mock.patch.object(corpus.Package, "reference", lambda self: "orig.cpp"), \
                mock.patch.object(corpus, "make_cases", made), \
                mock.patch.object(corpus, "cpl_cases", mock.Mock(side_effect=AssertionError("no port to test"))):
            self.assertEqual(corpus.check_tests(corpus.Package.at(d), ctx), [])
        made.assert_called_once()
        with mock.patch.object(corpus.Package, "reference", lambda self: "orig.cpp"), \
                mock.patch.object(corpus, "make_cases", mock.Mock(side_effect=corpus.ReferenceFails("03", "runs past 20 s"))):
            self.assertEqual(corpus.check_tests(corpus.Package.at(d), ctx), ["reference-fails q 03"])

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


    def test_a_reference_crash_on_a_made_case_is_a_finding(self):
        d = self.package("p", port_cp="print(1)\n", tests_toml="")
        with mock.patch.object(corpus.Package, "reference", lambda self: "orig.cpp"), \
                mock.patch.object(corpus, "make_cases", mock.Mock(side_effect=corpus.ReferenceFails("03", -11))):
            self.assertEqual(corpus.check_tests(corpus.Package.at(d), mock.Mock(cpl="cpl")), ["reference-fails p 03"])


class MakeTest(Corpus):
    def test_make_changed_makes_only_the_changed_packages(self):
        for pid in ("p1", "p2"):
            self.package(pid, tests_toml='[[case]]\ninput = "1"\n')
        made = []
        with mock.patch.object(corpus, "changed_ids", return_value=["p2", "gone"]), \
                mock.patch.object(corpus, "make_one", side_effect=lambda job: made.append(job[0])), \
                mock.patch("concurrent.futures.ProcessPoolExecutor", lambda *a, **k: mock.MagicMock(
                    __enter__=lambda s: type("P", (), {"map": staticmethod(lambda f, js: [f(j) for j in js])})(),
                    __exit__=lambda *a: False)):
            corpus.make(type("A", (), {"ids": [], "set": None, "out": None, "j": 1, "changed": "staged", "cpl": None}))
        self.assertEqual(made, ["p2"])

    def test_makes_every_package_in_worker_processes(self):
        for pid in ("p1", "p2", "p3"):
            self.package(pid, port_cp="", interactor_py="", tests_toml='[[case]]\ninput = "%s"\n' % pid)
        out = self.root / "out"
        args = mock.Mock(set=None, ids=[], out=str(out), j=2, changed=None, cpl="my-cpl", shard=None)

        def make_cases(pkg, out, cpl):
            Path(out, pkg.id).mkdir(parents=True)
            Path(out, pkg.id, "cpl").write_text(cpl)
        with mock.patch.object(corpus, "make_cases", make_cases):
            self.assertEqual(corpus.make(args), 0)
        self.assertEqual([(out / pid / "cpl").read_text() for pid in ("p1", "p2", "p3")], ["my-cpl"] * 3)

    def test_make_cases_is_cpl_test_make_and_names_a_failing_case(self):
        pkg = corpus.Package.at(self.package("p", tests_toml='[[case]]\ninput = "1"\n'))
        calls = []

        def run(argv, **kw):
            calls.append(argv)
            return mock.Mock(returncode=1, stderr="cpl: p: accepted C++ exits 3 on case 02\n")
        with mock.patch.object(corpus.subprocess, "run", run), self.assertRaises(corpus.ReferenceFails) as e:
            corpus.make_cases(pkg, "out", "my-cpl")
        self.assertEqual(calls, [["my-cpl", "test", "--make", str(pkg.dir), "--tests", os.path.join("out", "p")]])
        self.assertEqual(e.exception.case, "02")

    def test_a_reference_past_its_budget_fails_the_case_it_ran_on(self):
        pkg = corpus.Package.at(self.package("p", tests_toml='[[case]]\ninput = "1"\n'))
        failed = mock.Mock(returncode=1, stderr="cpl: p: accepted C++ runs past 20 s on case 03\n")
        with mock.patch.object(corpus.subprocess, "run", lambda argv, **kw: failed), \
                self.assertRaises(corpus.ReferenceFails) as e:
            corpus.make_cases(pkg, "out", "my-cpl")
        self.assertEqual((e.exception.case, str(e.exception)), ("03", "accepted C++ runs past 20 s on case 03"))


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


class BoundsTest(unittest.TestCase):
    """`// Bounds:` states the statement's input bounds; the read() types must take them all."""

    PORT = ("// Source: x\n// Bounds: n: 2..=1e5; a.x, a.y: 1..=1e9;\n//   q: 1..=n\n"
            "domain Q = 1..=n\nlet n: 2..=100_000 = read()  // items\nlet a: [n](x: 1..=1e9, y: {y}) = read()\n"
            "let q: Q = read()\n")

    def test_bounds_the_reads_take_are_clean(self):
        self.assertEqual(corpus.bounds_findings("p", self.PORT.format(y="1..=1e9")), [])

    def test_a_narrower_read_is_a_finding(self):
        self.assertEqual(corpus.bounds_findings("p", self.PORT.format(y="1..=1e6")),
                         ["bounds-narrow p a.y: reads 1..=1e6, statement 1..=1e9"])

    def test_a_wider_read_and_an_exclusive_end_pass(self):
        port = "// Bounds: n: 1..=10; k: 0..=n - 1\nlet n: 0..=1e9 = read()\nlet k: 0..n = read()\n"
        self.assertEqual(corpus.bounds_findings("p", port), [])

    def test_a_ranged_read_needs_the_header_and_every_read_in_it(self):
        self.assertEqual(corpus.bounds_findings("p", "let n: 1..=9 = read()\n"), ["no-header-bounds p -"])
        self.assertEqual(corpus.bounds_findings("p", "// Bounds: n: 1..=9\nlet n: 1..=9 = read()\nlet m: 1..=9 = read()\n"),
                         ["bounds-undeclared p m"])
        self.assertEqual(corpus.bounds_findings("p", "let n: Int = read()\n"), [])

    def test_a_read_of_lists_in_an_array_is_checked(self):
        got = corpus.port_ranges("type City = 1..=n\nlet roads: [0..n]List<(u: City, v: 1..=n)> = read()\n")
        self.assertEqual((got["roads.u"], got["roads.v"]), (("1", "n"), ("1", "n")))

    def test_an_enum_read_is_checked_by_its_variants_fields(self):
        got = corpus.port_ranges("domain W = 1..=n\nenum Op:\n    @tag(1) Join(u: W, v: W)\n    @tag(2) Split(u: W)\n"
                                 "    Ask(s: Str, k: 1..=50)\n    Stop\nlet ops = read<[q]Op>()\n")
        self.assertEqual(got, {"ops.u": ("1", "n"), "ops.v": ("1", "n"), "ops.k": ("1", "50")})

    def test_a_read_inside_an_expression_or_a_for_is_checked(self):
        got = corpus.port_ranges("let xs: List<P> = read<[c](x: 1..=n, y: 1..=m)>().sorted()\n"
                                 "for a in read<[0..m]1..=n>():\n    print(a)\nlet z = unread<1..=3>()\n")
        self.assertEqual(got, {"xs.x": ("1", "n"), "xs.y": ("1", "m"), "a": ("1", "n")})

    def test_a_read_fmt_wrapped_over_lines_is_checked(self):
        got = corpus.port_ranges("let trie = Rooted(\n    Graph.undirected(read<[k - 1](u: 1..=k, c: 1..=9)>()),\n"
                                 "    root: 1,\n)\nlet z: 1..=3 = read()\n")
        self.assertEqual(got, {"trie.u": ("1", "k"), "trie.c": ("1", "9"), "z": ("1", "3")})

    def test_a_string_reads_its_length_range_as_name_len(self):
        port = "let k: Str<1..=10000> = read()\nlet d: 1..=100 = read()\n"
        self.assertEqual(corpus.port_ranges(port), {"k.len": ("1", "10000"), "d": ("1", "100")})
        self.assertEqual(corpus.bounds_findings("p", "// Bounds: d: 1..=100\n" + port), ["bounds-undeclared p k.len"])
        self.assertEqual(corpus.bounds_findings("p", "// Bounds: k.len: 1..=1e4; d: 1..=100\n" + port), [])
        self.assertEqual(corpus.bounds_findings("p", "// Bounds: k.len: 1..=1e5; d: 1..=100\n" + port),
                         ["bounds-narrow p k.len: reads 1..=10000, statement 1..=1e5"])

    def test_an_unnamed_tuple_has_positional_fields_the_header_can_state_whole(self):
        port = "domain Col = 1..=w\ntype Cell = (Col, 0..n)\nlet s: [q](1..=3, Col) = read()\nlet c: Cell = read()\n"
        self.assertEqual(corpus.port_ranges(port), {"s.0": ("1", "3"), "s.1": ("1", "w"),
                                                    "c.0": ("1", "w"), "c.1": ("0", "n - 1")})
        self.assertEqual(corpus.bounds_findings("p", "// Bounds: s: [q](1..=3, Col); c.0: Col; c.1: 0..n\n" + port), [])
        self.assertEqual(corpus.bounds_findings("p", "// Bounds: s: [q](1..=3, 1..=2e5); c: Cell\n" + port),
                         ["bounds-narrow p s.1: reads 1..=w, statement 1..=2e5"])

    def test_a_read_of_a_range_or_a_parsed_line_is_checked(self):
        port = ("let n = read(1..=5000)\ndomain I = read(2..=2e5)\nlet a: [0..read(1..=10)]1..=9 = read()\n"
                "let m = read_line().parse<1..=100>()!\nprint(read(0..3))\n")
        self.assertEqual(corpus.port_ranges(port), {"n": ("1", "5000"), "I.count": ("2", "2e5"), "a.len": ("1", "10"),
                                                    "a": ("1", "9"), "m": ("1", "100"), "read(0..3)": ("0", "3 - 1")})
        self.assertEqual(corpus.bounds_findings("p", port), ["no-header-bounds p -"])
        const = "// Bounds: n: 2..=5e5\nconst N = 500_000\nlet n: 2..=N = read()\n"
        self.assertEqual(corpus.bounds_findings("p", const), [])
        self.assertEqual(corpus.bounds_findings("p", const.replace("5e5", "6e5")), ["bounds-narrow p n: reads 2..=N, statement 2..=6e5"])

    def test_bounds_end_at_the_next_header_key(self):
        text = "// Bounds: n: 1..=9\n// limits: 1000 ms, 256 MB\n//   q: 1..=5\nlet n: 1..=9 = read()\n"
        self.assertEqual(corpus.declared_bounds(text), {"n": ("1", "9")})

    def test_a_read_spelled_read_of_a_type_or_indented_is_checked(self):
        self.assertEqual(corpus.bounds_findings("p", "// Bounds: n: 1..=9\nlet n = read<1..=5>()\n"),
                         ["bounds-narrow p n: reads 1..=5, statement 1..=9"])
        self.assertEqual(corpus.bounds_findings("p", "// Bounds: k: 0..=1\nlet k: 0..=1 = read()\nif k:\n    let m: 1..=5 = read()\n"),
                         ["bounds-undeclared p m"])
        self.assertEqual(corpus.bounds_findings("p", "let v: List<Int> = read<List<Int>>()\n"), [])

    def test_a_read_through_a_type_alias_is_checked(self):
        port = ("// Bounds: weak: 1..=2e9; toys.weight, toys.size: 1..=2e9\ntype Limit = 1..=2e9\n"
                "let weak: [3]Limit = read()\nlet toys: [3](weight: Limit, size: Limit) = read()\n")
        self.assertEqual(corpus.bounds_findings("p", port), [])
        self.assertEqual(corpus.bounds_findings("p", port.replace("type Limit = 1..=2e9", "type Limit = 1..=1e9")),
                         ["bounds-narrow p weak: reads 1..=1e9, statement 1..=2e9",
                          "bounds-narrow p toys.weight: reads 1..=1e9, statement 1..=2e9",
                          "bounds-narrow p toys.size: reads 1..=1e9, statement 1..=2e9"])
        self.assertEqual(corpus.bounds_findings("p", "// Bounds: qs.t: 0..=1e9; a: 0..=2^30\nstruct Query(t: 0..=1e9, i: Pos)\n"
                                                     "type Coord = 0..=2^30\nlet qs: [Q]Query = read()\nlet a: [n]Coord = read()\n"), [])

    def test_a_second_bounds_line_is_a_finding(self):
        self.assertEqual(corpus.bounds_findings("p", "// Bounds: n: 1..=9\n// Bounds: n: 1..=9\nlet n: 1..=9 = read()\n"),
                         ["bounds-twice p -"])

    def test_a_stated_bound_nothing_reads_is_a_finding(self):
        self.assertEqual(corpus.bounds_findings("p", "// Bounds: n: 1..=9; m\nlet n: 1..=9 = read()\n"),
                         ["bounds-unparsed p m"])
        self.assertEqual(corpus.bounds_findings("p", "// Bounds: n, m: 1..=9\nlet n: 1..=9 = read()\n"),
                         ["bounds-unread p m"])


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
        limits = tuple(corpus.package_limits(corpus.Package.at(d)))
        self.assertEqual(a, ("--tests", str(cases), "--interactor", str(d / "interactor.py"), *limits))
        self.assertEqual(limits[0], "--time-limit")

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

    def test_an_inline_input_over_64k_is_a_finding_and_an_input_file_is_not(self):
        big = "1 " * (32 * 1024 + 1)
        d = self.package("p1", port_cp="", tests_toml=f'[[case]]\ninput = "1"\n\n[[case]]\ninput = "{big}"\n\n'
                                                       '[[case]]\ninput_file = "samples/3.in"\n')
        (d / "samples").mkdir()
        (d / "samples" / "3.in").write_text(big)
        found = corpus.check_package(corpus.Package.at(d), self.ctx(""))
        self.assertEqual([f for f in found if f.startswith("inline-input")],
                         ["inline-input-over-64k p1 case=2,use=input_file"])

    def test_committed_tests_are_a_finding(self):
        d = self.package("p1", port_cp="")
        (d / "tests").mkdir()
        self.assertIn("committed-tests p1 -", corpus.check_package(corpus.Package.at(d), self.ctx("")))

    def test_a_package_with_cases_needs_a_reference(self):
        pkg = corpus.Package.at(self.package("p1", port_cp="", tests_toml='[[case]]\ninput = "1"\n', meta_json="{}"))
        self.assertEqual(corpus.check_package(pkg, self.ctx("")), ["no-reference p1 -"])


    def test_orig_cpp_alone_is_the_reference(self):
        sub = self.root / "narrow.cpp"
        sub.write_text("int main(){}\n")
        meta = '{"submissions": [{"file": "%s", "result": "AC"}]}' % sub
        pkg = corpus.Package.at(self.package("p1", orig_cpp="int main(){}\n", meta_json=meta))
        self.assertEqual(corpus.accepted_cpps(pkg), [pkg.orig])
        pkg2 = corpus.Package.at(self.package("p2", meta_json=meta))
        self.assertEqual(corpus.accepted_cpps(pkg2), [sub])

    def test_a_reference_that_changes_on_a_rerun_does_not_ask_for_a_checker(self):
        refs = [self.root / "a.cpp", self.root / "b.cpp", self.root / "c.cpp"]
        for r in refs:
            r.write_text("int main(){}\n")
        pkg = corpus.Package.at(self.package("p1", tests_toml='[[case]]\ninput = "1"\n', meta_json="{}"))
        flips = iter([b"no", b"yes"] * 4)
        outs = {"a": lambda: b"yes", "b": lambda: next(flips), "c": lambda: b"yes"}
        ctx = self.ctx("")
        ctx.run = True
        with mock.patch.object(corpus, "accepted_cpps", return_value=refs), mock.patch.object(corpus, "make_cases"), \
                mock.patch.object(corpus, "generated", return_value=[mock.Mock(read_bytes=lambda: b"1\n")]), \
                mock.patch.object(corpus, "built", side_effect=lambda src: src.stem), \
                mock.patch.object(corpus, "run_out", side_effect=lambda e, d: outs[e]()):
            self.assertEqual(corpus.check_package(pkg, ctx), [])
            outs["c"] = lambda: b"no"
            self.assertEqual(corpus.check_package(pkg, ctx), ["needs-checker p1 1"])
            outs["c"] = lambda: None
            self.assertEqual(corpus.check_package(pkg, ctx), ["reference-fails p1 1"])

class LintModeTest(Corpus):
    def ctx(self, migrate, fmt):
        cpl = self.root / "cpl"
        cpl.write_text("#!/bin/sh\ncase $1 in migrate) echo '%s';; fmt) echo '%s';; esac\n" % (migrate, fmt))
        cpl.chmod(0o755)
        return corpus.Context(str(cpl), run=False)

    def files(self, *changed):
        return json.dumps({"files": [{"path": f"x/{name}", "changed": c, "diagnostics": []} for name, c in changed]})

    def test_a_file_fmt_would_change_is_a_finding(self):
        pkg = corpus.Package.at(self.package("p1", port_cp="", dream_cp=""))
        ctx = self.ctx(self.files(("port.cp", False), ("dream.cp", False)),
                       self.files(("port.cp", False), ("dream.cp", True)))
        self.assertEqual(corpus.check_lint(pkg, ctx), ["fmt-dirty p1 dream.cp"])

    def test_formatted_files_with_current_spellings_pass(self):
        pkg = corpus.Package.at(self.package("p1", port_cp=""))
        ctx = self.ctx(self.files(("port.cp", False)), self.files(("port.cp", False)))
        self.assertEqual(corpus.check_lint(pkg, ctx), [])

    def test_unreadable_fmt_output_is_a_finding(self):
        pkg = corpus.Package.at(self.package("p1", port_cp=""))
        self.assertEqual(corpus.check_lint(pkg, self.ctx(self.files(("port.cp", False)), "oops")), ["fmt-error p1 -"])


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
        cases = tomllib.loads((self.dir / "tests.toml").read_text())["case"]
        self.assertEqual(cases[0], {"input": "tiny\n"})
        self.assertEqual(cases[1], {"seed": 2, "size": 16})
        self.assertEqual([c["size"] for c in cases[2:]], list(corpus.STRESS_SIZES))

    def test_an_inline_case_over_64k_is_seeded_as_an_input_file(self):
        d = self.package("p3", interactor_py="")
        (d / "tests").mkdir()
        big = "x" * (corpus.INLINE_INPUT_LIMIT + 1)
        (d / "tests" / "1.in").write_text(big)
        corpus.seed(type("A", (), {"ids": ["p3"]}))
        cases = tomllib.loads((d / "tests.toml").read_text())["case"]
        self.assertEqual(cases, [{"input_file": "samples/1.in"}])
        self.assertEqual((d / "samples" / "1.in").read_text(), big)

    def test_a_large_case_without_a_generator_is_not_seeded(self):
        d = self.package("p2")
        (d / "tests").mkdir()
        (d / "tests" / "1.in").write_text("x" * (corpus.INLINE + 1))
        self.assertEqual(corpus.seed(type("A", (), {"ids": ["p2"]})), 1)
        self.assertTrue((d / "tests").exists())


class NativeTest(Corpus):
    """`native` builds the port for the judge its package's cp.toml names, as `cpl test` does."""

    def judge_built(self, d):
        seen = []
        ctx = types.SimpleNamespace(cpl="cpl", natives={})
        with mock.patch.object(corpus.subprocess, "run", lambda cmd, **kw: seen.append(cmd) or types.SimpleNamespace(returncode=1)):
            self.assertEqual(corpus.native(corpus.Package.at(d), ctx), (None, 0))
        return seen[0][seen[0].index("--judge") + 1]

    def test_the_packages_judge_reaches_the_build(self):
        toml = '[judge]\njudge = "noi"\n\n[judge.noi]\nextends = "generic"\nisa = "avx2"\n'
        self.assertEqual(self.judge_built(self.package("p1", port_cp="", cp_toml=toml)), "noi")

    def test_a_package_without_one_builds_for_codeforces(self):
        self.assertEqual(self.judge_built(self.package("p2", port_cp="")), "codeforces")
        self.assertEqual(self.judge_built(self.package("p3", port_cp="", cp_toml="[libs]\n")), "codeforces")


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

    def test_a_checked_run_past_its_budget_is_not_called_a_trap(self):
        self.assertEqual(self.findings(None, (0, b"1\n", 0.05, 10)), ["checked-over-limit p --max"])

    def test_over_the_time_limit(self):
        self.assertIn("over-limit p --max", self.findings((0, b"1\n", 1, 10), (0, b"1\n", 2.5, 10), (0, b"1\n", 2.4, 10)))

    def test_a_box_slower_than_the_judge_measures_against_the_accepted_cpp(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), (0, b"1\n", 2.2, 10), (0, b"1\n", 2.4, 10)), [])
        self.assertIn("over-limit p --max", self.findings((0, b"1\n", 1, 10), (0, b"1\n", 2.5, 10), (0, b"1\n", 2.4, 10)))

    def test_over_the_memory_limit(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), (0, b"1\n", 0.05, 300)), ["over-limit p --max"])

    def test_a_judge_timeout_is_over_limit(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), None), ["over-limit p --max"])

    def test_a_different_answer_disagrees(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), (0, b"2\n", 0.05, 10)), ["disagrees p --max"])

    def test_slow_against_the_accepted_cpp(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), (0, b"1\n", 0.5, 10), (0, b"1\n", 0.2, 10)), ["slow p --max"])

    def test_an_accepted_cpp_that_fails_the_max_is_a_finding(self):
        self.assertEqual(self.findings((0, b"1\n", 1, 10), (0, b"1\n", 0.05, 10), (139, b"", 0.01, 10)), ["reference-fails p --max"])
        self.assertEqual(self.findings((0, b"1\n", 1, 10), (0, b"1\n", 0.05, 10), None), ["reference-fails p --max"])


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


class DeleteTest(Corpus):
    def test_removes_the_package_its_submissions_and_every_line_naming_it(self):
        subs, unknowns = self.root / "subs", self.root / "silent_unknowns.txt"
        subs.mkdir()
        for name, value in (("SUBS", subs), ("UNKNOWNS", unknowns)):
            self.addCleanup(setattr, corpus, name, getattr(corpus, name))
            setattr(corpus, name, str(value))
        self.package("p2", port_cp="")
        self.package("p3", port_cp="")
        info = {"910000001": {"problem": "p2"}, "910000002": {"problem": "p3"}}
        (subs / "info.json").write_text(json.dumps(info, indent=4))
        (subs / "910000001.cpp").write_text("")
        (subs / "910000002.cpp").write_text("")
        unknowns.write_text(f"1 {PKGS}/p2/port.cp\n1 {PKGS}/p3/port.cp\n")
        build = mock.Mock(return_value=0)
        with mock.patch.object(corpus, "main", build), mock.patch.object(corpus, "named_by", lambda *_: []), \
                contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(corpus.delete(["p2"]), 0)
        build.assert_called_once_with("build")
        self.assertEqual([p.id for p in corpus.problems()], ["p3"])
        self.assertEqual(json.loads((subs / "info.json").read_text()), {"910000002": {"problem": "p3"}})
        self.assertEqual(sorted(f.name for f in subs.iterdir()), ["910000002.cpp", "info.json"])
        self.assertEqual((self.root / "sets" / "small.txt").read_text(), "# comment\n")
        self.assertEqual(unknowns.read_text(), f"1 {PKGS}/p3/port.cp\n")

    def test_a_submission_another_file_names_is_kept_without_its_entry(self):
        subs = self.root / "subs"
        subs.mkdir()
        self.addCleanup(setattr, corpus, "SUBS", corpus.SUBS)
        corpus.SUBS = str(subs)
        self.package("p2")
        (subs / "info.json").write_text(json.dumps({"910000001": {"problem": "p2"}}, indent=4))
        (subs / "910000001.cpp").write_text("")
        out = io.StringIO()
        with mock.patch.object(corpus, "main", mock.Mock(return_value=0)), \
                mock.patch.object(corpus, "named_by", lambda *_: ["bench/audit.py"]), contextlib.redirect_stdout(out):
            corpus.delete(["p2"])
        self.assertTrue((subs / "910000001.cpp").exists())
        self.assertEqual(json.loads((subs / "info.json").read_text()), {})
        self.assertIn("named by bench/audit.py", out.getvalue())

    def test_its_v02_snapshot_goes_and_a_path_still_naming_it_fails_the_deletion(self):
        subs = self.root / "subs"
        subs.mkdir()
        for name, value in (("ROOT", self.root), ("SUBS", subs)):
            self.addCleanup(setattr, corpus, name, getattr(corpus, name))
            setattr(corpus, name, str(value))
        self.package("p2")
        self.package("p3")
        (subs / "info.json").write_text(json.dumps({"910000001": {"problem": "p2"}}, indent=4))
        (subs / "910000001.cpp").write_text("")
        for pid in ("p2", "p3"):
            (self.root / "tests" / "v02-snapshot" / pid).mkdir(parents=True)
            (self.root / "tests" / "v02-snapshot" / pid / "port.cp").write_text("")
        (self.root / "bench").mkdir()
        (self.root / "bench" / "audit.py").write_text(f"{PKGS}/p2/port.cp\n{PKGS}/p3/port.cp\n"
                                                      f"{SUBS_DIR}/910000001.cpp\n")
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True)
        subprocess.run(["git", "add", "bench/audit.py"], cwd=self.root, check=True)
        out = io.StringIO()
        with mock.patch.object(corpus, "main", mock.Mock(return_value=0)), \
                mock.patch.object(corpus, "named_by", lambda *_: []), contextlib.redirect_stdout(out):
            self.assertEqual(corpus.delete(["p2"]), 1)
        self.assertEqual(sorted(p.name for p in (self.root / "tests" / "v02-snapshot").iterdir()), ["p3"])
        self.assertIn(f"bench/audit.py:1: still names deleted {PKGS}/p2", out.getvalue())
        self.assertIn(f"bench/audit.py:3: still names deleted {SUBS_DIR}/910000001.cpp", out.getvalue())
        self.assertNotIn("p3", out.getvalue())

    def test_a_deletion_nothing_else_names_succeeds(self):
        subs = self.root / "subs"
        subs.mkdir()
        for name, value in (("ROOT", self.root), ("SUBS", subs)):
            self.addCleanup(setattr, corpus, name, getattr(corpus, name))
            setattr(corpus, name, str(value))
        self.package("p2")
        (subs / "info.json").write_text("{}")
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True)
        with mock.patch.object(corpus, "main", mock.Mock(return_value=0)), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(corpus.delete(["p2"]), 0)

    def test_a_hill_climb_problem_is_refused_before_anything_changes(self):
        subs = self.root / "subs"
        subs.mkdir()
        for name, value in (("ROOT", self.root), ("SUBS", subs)):
            self.addCleanup(setattr, corpus, name, getattr(corpus, name))
            setattr(corpus, name, str(value))
        (self.root / "corpus").mkdir()
        (self.root / "corpus" / "hillclimb.json").write_text(json.dumps({"problems": ["p3"]}))
        self.package("p2", port_cp="")
        self.package("p3", port_cp="")
        info = json.dumps({"910000001": {"problem": "p2"}, "910000002": {"problem": "p3"}}, indent=4)
        (subs / "info.json").write_text(info)
        (subs / "910000002.cpp").write_text("")
        before = sorted(str(p.relative_to(self.root)) for p in self.root.rglob("*"))
        with mock.patch.object(corpus, "main") as build, self.assertRaises(SystemExit) as e:
            corpus.delete(["p2", "p3"])
        self.assertIn("p3 is in corpus/hillclimb.json", str(e.exception))
        build.assert_not_called()
        self.assertEqual(sorted(str(p.relative_to(self.root)) for p in self.root.rglob("*")), before)
        self.assertEqual((subs / "info.json").read_text(), info)

    def test_an_unknown_id_stops_before_anything_is_removed(self):
        with self.assertRaises(SystemExit) as e:
            corpus.delete(["nope"])
        self.assertIn("no package nope", str(e.exception))


class VerifyCplTest(Corpus):
    def test_a_missing_cpl_says_to_build_it(self):
        d = self.package("p1", gen_py=GEN)
        args = argparse.Namespace(ids=["p1"], cpl=str(self.root / "no-cpl"), only="tests")
        with mock.patch.object(corpus, "problems", lambda sets=None, ids=None: iter([corpus.Package.at(d)])), \
                mock.patch.object(corpus, "require_cpl", REQUIRE_CPL), self.assertRaises(SystemExit) as e:
            corpus.verify(args)
        self.assertIn("cargo build --release -p cpl-cli", str(e.exception))

    def test_the_package_mode_needs_a_cpl_too(self):
        # package makes the cases with `cpl test --make` to check the seeds and references.
        d = self.package("p1", gen_py=GEN)
        args = argparse.Namespace(ids=["p1"], cpl=str(self.root / "no-cpl"), only="package")
        with mock.patch.object(corpus, "problems", lambda sets=None, ids=None: iter([corpus.Package.at(d)])), \
                mock.patch.object(corpus, "require_cpl", REQUIRE_CPL), self.assertRaises(SystemExit) as e:
            corpus.verify(args)
        self.assertIn("cargo build --release -p cpl-cli", str(e.exception))


class StaleCplTest(unittest.TestCase):
    """A `cpl` under target/ older than a source it is built from is stale."""

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        self.addCleanup(setattr, corpus, "ROOT", corpus.ROOT)
        corpus.ROOT = str(self.root)
        for path, text in (("Cargo.toml", "[workspace]\n"),
                           ("crates/cpl-cli/Cargo.toml", "[dependencies]\ncpl-a.workspace = true\n"),
                           ("crates/cpl-cli/src/main.rs", ""),
                           ("crates/cpl-a/Cargo.toml", "[dependencies]\n"),
                           ("crates/cpl-a/src/lib.rs", ""),
                           ("crates/cpl-other/Cargo.toml", "[dependencies]\n"),
                           ("crates/cpl-other/src/lib.rs", ""),
                           ("target/release/cpl", "")):
            (self.root / path).parent.mkdir(parents=True, exist_ok=True)
            (self.root / path).write_text(text)
        self.cpl = self.root / "target/release/cpl"
        for f in self.root.rglob("*"):
            os.utime(f, (1000, 1000))
        os.utime(self.cpl, (2000, 2000))

    def stale(self, cpl=None):
        with self.assertRaises(SystemExit) as e:
            corpus.require_cpl(str(cpl or self.cpl))
        return str(e.exception)

    def test_a_cpl_newer_than_its_sources_runs(self):
        corpus.require_cpl(str(self.cpl))

    def test_a_cpl_older_than_a_dependency_source_says_to_rebuild_it(self):
        os.utime(self.root / "crates/cpl-a/src/lib.rs", (3000, 3000))
        self.assertEqual(self.stale(), "corpus: target/release/cpl is stale: run cargo build --release -p cpl-cli")

    def test_a_crate_cpl_does_not_depend_on_leaves_it_fresh(self):
        os.utime(self.root / "crates/cpl-other/src/lib.rs", (3000, 3000))
        corpus.require_cpl(str(self.cpl))

    def dep_info(self, *deps):
        Path(f"{self.cpl}.d").write_text(f"{self.cpl}: " + " ".join(str(self.root / d).replace(" ", "\\ ") for d in deps) + "\n")

    def test_cargo_dep_info_names_inputs_outside_the_crates(self):
        (self.root / "std").mkdir()
        (self.root / "std/a b.cp").write_text("")
        os.utime(self.root / "std/a b.cp", (1000, 1000))
        self.dep_info("crates/cpl-cli/src/main.rs", "std/a b.cp")
        corpus.require_cpl(str(self.cpl))
        os.utime(self.root / "std/a b.cp", (3000, 3000))
        self.assertEqual(self.stale(), "corpus: target/release/cpl is stale: run cargo build --release -p cpl-cli")

    def test_cargo_dep_info_is_the_whole_list(self):
        self.dep_info("crates/cpl-cli/src/main.rs")
        os.utime(self.root / "crates/cpl-a/src/lib.rs", (3000, 3000))
        corpus.require_cpl(str(self.cpl))

    def test_a_newer_git_head_only_restamps_the_version_so_cpl_stays_fresh(self):
        (self.root / ".git").mkdir()
        (self.root / ".git/HEAD").write_text("")
        os.utime(self.root / ".git/HEAD", (3000, 3000))
        self.dep_info("crates/cpl-cli/src/main.rs", ".git/HEAD")
        corpus.require_cpl(str(self.cpl))

    def test_a_dependency_cargo_recorded_but_is_gone_is_stale(self):
        self.dep_info("crates/cpl-cli/src/main.rs", "crates/cpl-cli/src/gone.rs")
        self.assertIn("is stale", self.stale())

    def test_a_cpl_outside_target_is_never_stale(self):
        cpl = self.root / "elsewhere-cpl"
        cpl.write_text("")
        os.utime(cpl, (500, 500))
        corpus.require_cpl(str(cpl))

    def test_a_missing_cpl_says_to_build_it(self):
        self.assertIn("cargo build --release -p cpl-cli", self.stale(self.root / "target/release/none"))

    def test_verify_stops_on_a_stale_cpl_before_running_anything(self):
        os.utime(self.root / "crates/cpl-cli/src/main.rs", (3000, 3000))
        args = argparse.Namespace(ids=["p1"], cpl=None, only="tests")
        pkg = types.SimpleNamespace(id="p1")
        with mock.patch.object(corpus, "problems", lambda sets=None, ids=None: iter([pkg])), \
                mock.patch.object(corpus, "findings", side_effect=AssertionError("ran a check")), \
                self.assertRaises(SystemExit) as e:
            corpus.verify(args)
        self.assertEqual(str(e.exception), "corpus: target/release/cpl is stale: run cargo build --release -p cpl-cli")

    def test_make_stops_on_a_stale_cpl_before_running_anything(self):
        os.utime(self.root / "Cargo.toml", (3000, 3000))
        args = argparse.Namespace(ids=["p1"], cpl=None, set=None, out=None, j=1)
        pkg = types.SimpleNamespace(id="p1", tests=True)
        with mock.patch.object(corpus, "problems", lambda sets=None, ids=None: iter([pkg])), \
                mock.patch.dict(os.environ, {"CPL": ""}), \
                mock.patch.object(corpus, "make_one", side_effect=AssertionError("made a case")), \
                self.assertRaises(SystemExit) as e:
            corpus.make(args)
        self.assertEqual(str(e.exception), "corpus: target/release/cpl is stale: run cargo build --release -p cpl-cli")

class GenTimeoutTest(unittest.TestCase):
    def test_a_max_generator_gets_the_briefs_60_seconds(self):
        pkg = mock.Mock()
        pkg.gen_input.return_value = "1\n"
        corpus.gen_out(pkg, ["--max"])
        self.assertEqual(pkg.gen_input.call_args.kwargs["timeout"], 60)


class GraderTest(Corpus):
    def test_a_program_of_grader_fns_is_linked_with_the_packages_grader(self):
        d = self.package("p1", port_cp="// the judge form is `@grader fn f`\nprint(1)\n", dream_cp="@grader fn f(n: Int) -> Int = n\n", checker_py="")
        (d / "dream.tests").mkdir()
        (d / "dream.tests" / "grader.cpp").write_text("int main(){}\n")
        pkg = corpus.Package.at(d)
        self.assertEqual(corpus.judge_args(pkg, pkg.dream),
                         ["--checker", str(d / "checker.py"), "--grader", str(d / "dream.tests" / "grader.cpp")])
        self.assertEqual(corpus.judge_args(pkg, pkg.port), ["--checker", str(d / "checker.py")])


class DanglingReferenceTest(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        for name, value in (("ROOT", self.root), ("PROBLEMS", self.root / PKGS), ("SUBS", self.root / SUBS_DIR)):
            self.addCleanup(setattr, corpus, name, getattr(corpus, name))
            setattr(corpus, name, str(value))
        (self.root / PKGS / "p1").mkdir(parents=True)
        (self.root / SUBS_DIR).mkdir(parents=True)
        (self.root / SUBS_DIR / "1.cpp").write_text("")
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True)

    def track(self, path, text):
        (self.root / path).parent.mkdir(parents=True, exist_ok=True)
        (self.root / path).write_text(text)
        subprocess.run(["git", "add", path], cwd=self.root, check=True)

    def test_a_path_to_a_missing_package_or_submission_is_named(self):
        self.track("bench/audit.py", f"{PKGS}/p1/port.cp\n{PKGS}/gone/port.cp\n"
                                     f"{SUBS_DIR}/1.cpp\n'{SUBS_DIR}/2.cpp'.\n")
        self.assertEqual(corpus.dangling_references(), [("bench/audit.py", 2, f"{PKGS}/gone"),
                                                        ("bench/audit.py", 4, f"{SUBS_DIR}/2.cpp")])

    def test_existing_targets_prefixes_placeholders_and_foreign_roots_pass(self):
        self.track("docs/a.md", f"`{PKGS}/p1/`, {PKGS}/p*, {PKGS}/<id>, {SUBS_DIR}/1.cpp,\n"
                                f"{PKGS}/{{pid}}/x, /tmp/x/{PKGS}/gone, ../{SUBS_DIR}/9.cpp\n")
        self.assertEqual(corpus.dangling_references(), [])

    def test_a_prefix_nothing_matches_is_named(self):
        self.track("docs/a.md", f"{PKGS}/q*\n")
        self.assertEqual(corpus.dangling_references(), [("docs/a.md", 1, f"{PKGS}/q*")])

    def test_only_tracked_files_outside_corpus_count(self):
        self.track(f"{PKGS}/p1/notes.md", f"{PKGS}/gone\n")
        (self.root / "loose.md").write_text(f"{PKGS}/gone\n")
        self.assertEqual(corpus.dangling_references(), [])

    def test_a_script_tests_fixture_is_not_a_reference(self):
        self.track("scripts/test_x.py", f"{PKGS}/gone\n")
        self.track("scripts/x.py", f"{PKGS}/gone\n")
        self.assertEqual(corpus.dangling_references(), [("scripts/x.py", 1, f"{PKGS}/gone")])

    def test_check_fails_on_a_dangling_reference(self):
        err = io.StringIO()
        with mock.patch.object(corpus, "generate", return_value={}), mock.patch.object(corpus, "problems", return_value=[]), \
                mock.patch.object(corpus, "orphan_submissions", return_value=[]), \
                mock.patch.object(corpus, "dangling_references", return_value=[("bench/a.py", 3, f"{PKGS}/gone")]), \
                contextlib.redirect_stderr(err), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(corpus.main("build"), 1)
        self.assertIn(f"bench/a.py:3: names {PKGS}/gone, which doesn't exist", err.getvalue())



class MetaFormatTest(unittest.TestCase):
    GEN = {"problem": "p", "source": "x",
           "submissions": [{"id": "1", "language": "CPP17", "result": "AC", "time": 0.5, "memory_kb": 4.0, "file": None},
                           {"id": "2", "language": "PY3", "result": "WA", "time": 1.0, "memory_kb": 8.0, "file": None}],
           "cpp_ac": 1, "artifacts": {"original": [f"{PKGS}/p/orig.cpp"]}}

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        for name, value in (("ROOT", self.root), ("PROBLEMS", self.root / PKGS), ("SUBS", self.root / SUBS_DIR)):
            self.addCleanup(setattr, corpus, name, getattr(corpus, name))
            setattr(corpus, name, str(value))
        (self.root / PKGS / "p").mkdir(parents=True)
        (self.root / PKGS / "p" / "orig.cpp").write_text("")
        self.path = self.root / PKGS / "p" / "meta.json"
        for name, value in (("generate", {"p": self.GEN}), ("orphan_submissions", []), ("dangling_references", [])):
            patcher = mock.patch.object(corpus, name, return_value=value)
            patcher.start()
            self.addCleanup(patcher.stop)

    def run_main(self, mode):
        err = io.StringIO()
        with contextlib.redirect_stderr(err), contextlib.redirect_stdout(io.StringIO()):
            code = corpus.main(mode)
        return code, err.getvalue()

    def canonical(self, **hand):
        return corpus.meta_text({**self.GEN, **corpus.HAND, **hand})

    def test_meta_text_keeps_each_submission_row_on_one_line(self):
        self.assertEqual(corpus.meta_text({"problem": "p", "submissions": self.GEN["submissions"],
                                           "artifacts": {"original": ["a.cpp"]}, "tags": [], "notes": "\u00e9"}),
                         '{\n "problem": "p",\n "submissions": [\n'
                         '  {"id": "1", "language": "CPP17", "result": "AC", "time": 0.5, "memory_kb": 4.0, "file": null},\n'
                         '  {"id": "2", "language": "PY3", "result": "WA", "time": 1.0, "memory_kb": 8.0, "file": null}\n ],\n'
                         ' "artifacts": {\n  "original": [\n   "a.cpp"\n  ]\n },\n "tags": [],\n "notes": "\\u00e9"\n}\n')

    def test_every_committed_meta_json_is_already_canonical(self):
        for d in sorted(Path(corpus.__file__).resolve().parent.parent.joinpath(PKGS).iterdir()):
            if (d / "meta.json").is_file():
                text = (d / "meta.json").read_text()
                self.assertEqual(text, corpus.meta_text(json.loads(text)), d.name)

    def test_check_fails_a_json_dump_edit_and_build_restores_the_canonical_bytes(self):
        want = self.canonical(notes="caf\u00e9")
        self.path.write_text(json.dumps(json.loads(want), indent=2, ensure_ascii=False))
        code, err = self.run_main("check")
        self.assertEqual(code, 1)
        self.assertIn(f"{PKGS}/p/meta.json: not in canonical form; run scripts/corpus.py build", err)
        self.assertEqual(self.run_main("build"), (0, ""))
        self.assertEqual(self.path.read_text(), want)
        self.assertEqual(self.run_main("check"), (0, ""))

    def test_a_wrong_generated_field_is_stale(self):
        self.path.write_text(self.canonical().replace('"cpp_ac": 1', '"cpp_ac": 2'))
        code, err = self.run_main("check")
        self.assertEqual(code, 1)
        self.assertIn(f"{PKGS}/p/meta.json: stale; run scripts/corpus.py build", err)

    def test_meta_json_that_isnt_json_is_named_and_build_leaves_it(self):
        self.path.write_text("{")
        for mode in ("check", "build"):
            code, err = self.run_main(mode)
            self.assertEqual(code, 1)
            self.assertIn(f"{PKGS}/p/meta.json: not JSON", err)
        self.assertEqual(self.path.read_text(), "{")

    def test_record_writes_the_canonical_form(self):
        self.path.write_text(json.dumps({**self.GEN, **corpus.HAND}))
        corpus.record(corpus.Package.at(self.path.parent), "dream_stress", {"ok": True, "cases": 8, "sha": "abc"})
        self.assertEqual(self.path.read_text(), self.canonical(dream_stress={"ok": True, "cases": 8, "sha": "abc"}))

class OrphanSubmissionTest(unittest.TestCase):
    def test_a_submission_file_no_package_lists_is_an_orphan(self):
        with tempfile.TemporaryDirectory() as d, mock.patch.object(corpus, "SUBS", d), \
                mock.patch.object(corpus.subprocess, "run", return_value=mock.Mock(returncode=1)):
            for n in ("1.cpp", "2.txt", "info.json"):
                Path(d, n).write_text("")
            metas = {"p": {"submissions": [{"file": f"{SUBS_DIR}/1.cpp"}, {"file": None}]}}
            self.assertEqual(corpus.orphan_submissions(metas), ["2.txt"])

    def test_a_file_another_tracked_file_names_is_kept(self):
        with tempfile.TemporaryDirectory() as d, mock.patch.object(corpus, "SUBS", d), \
                mock.patch.object(corpus.subprocess, "run", return_value=mock.Mock(returncode=0)):
            Path(d, "2.txt").write_text("")
            self.assertEqual(corpus.orphan_submissions({}), [])


if __name__ == "__main__":
    unittest.main()
