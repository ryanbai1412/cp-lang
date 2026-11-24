import importlib.util
import os
import stat
import subprocess
import sys
import tempfile
import unittest

spec = importlib.util.spec_from_file_location(
    "v03_perf_bench", os.path.join(os.path.dirname(__file__), "v03_perf_bench.py"))
bench = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bench)


def row(ratio, tuned=None):
    r = {"status": "ok", "port_ms": 100.0 * ratio, "orig_ms": 100.0, "ratio": ratio}
    return {**r, "tuned_ratio": tuned} if tuned else r


class Summarize(unittest.TestCase):
    def test_every_timed_pair_counts_in_the_ratios(self):
        st = bench.summarize([row(1.0), row(6.0)], min_ms=10)
        self.assertEqual(st["measured_ge_min_ms"], 2)
        self.assertEqual(st["max"], 6.0)

    def test_a_tuned_variant_lowers_only_its_own_pair(self):
        st = bench.summarize([row(1.0), row(6.0, tuned=0.8)], min_ms=10)
        self.assertEqual(st["max"], 6.0)
        self.assertEqual(st["tuned_max"], 1.0)

    def test_every_pair_without_a_ratio_counts_as_skipped(self):
        st = bench.summarize([row(1.0), {"status": "skip"}, {"status": "mismatch"}], min_ms=10)
        self.assertEqual((st["total_ok"], st["skipped"]), (1, 2))

    def test_there_is_no_unmatched_list(self):
        self.assertFalse(os.path.exists(os.path.join(bench.ROOT, "bench", "unmatched.txt")))


class CollectItems(unittest.TestCase):
    def cp_of(self, **kw):
        [item] = bench.collect_items({"corpus"}, {"mccc6s3"}, "/nonexistent", **kw)
        return os.path.relpath(item[2], bench.ROOT)

    def test_the_series_times_the_frozen_copy(self):
        self.assertEqual(self.cp_of(), "tests/v02-snapshot/mccc6s3/port.cp")

    def test_live_times_the_port_as_it_is_now(self):
        self.assertEqual(self.cp_of(live=True), "corpus/problems/mccc6s3/port.cp")

    def test_a_package_without_orig_cpp_is_timed_against_its_ac_submission(self):
        [item] = bench.collect_items({"corpus"}, {"996warmup8p3"}, "/nonexistent", live=True)
        self.assertEqual(os.path.relpath(item[3], bench.ROOT), "corpus/submissions/5324670.cpp")

    def test_the_reference_is_the_ac_submission_the_port_names_as_its_original(self):
        [item] = bench.collect_items({"corpus"}, {"noi02p2"}, "/nonexistent", live=True)
        self.assertEqual(os.path.relpath(item[3], bench.ROOT), "corpus/submissions/4793917.cpp")


def write(path, text, mode=0o644):
    with open(path, "w") as f:
        f.write(text)
    os.chmod(path, mode)
    return path


class CorpusInputs(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()

    def inputs(self, gen_max, spec_text):
        write(os.path.join(self.tmp, "gen.py"), f"import sys\nif sys.argv[1:] == ['--max']:\n    print({gen_max!r}, end='')\n")
        spec = (None, None, None, None, lambda out, big, _: write(out, spec_text))
        cache = tempfile.mkdtemp()
        return [open(f).read() for f in bench.corpus_inputs("pkg", self.tmp, spec, cache)]

    def test_the_packages_own_max_input_comes_before_the_bench_generator(self):
        self.assertEqual(self.inputs("from gen.py\n", "from the bench\n"), ["from gen.py\n"])

    def test_an_empty_max_input_is_an_input(self):
        self.assertEqual(self.inputs("", "from the bench\n"), [""])


INTERACTOR = f"""import os, sys
sys.path.insert(0, {os.path.join(bench.ROOT, "corpus", "lib")!r})
from interactor_lib import run_interactor
def drive(case_text, io):
    io.send(case_text.split()[0])
    if io.recv_line().split() != ["!", case_text.split()[0]]:
        raise RuntimeError("wrong")
run_interactor(drive)
"""


class RunInteractive(unittest.TestCase):
    def run_with(self, solution, spawn=False):
        d = tempfile.mkdtemp()
        judge = write(os.path.join(d, "interactor.py"), ("# interactor: spawn\n" if spawn else "") + INTERACTOR)
        sol = write(os.path.join(d, "sol"), f"#!{sys.executable}\n{solution}", 0o755)
        case = write(os.path.join(d, "case"), "7\n")
        return bench.run_interactive(sol, judge, case, 20)[1]

    def test_an_accepted_run_is_ok_over_pipes(self):
        self.assertEqual(self.run_with("print('!', input(), flush=True)"), "ok")

    def test_an_accepted_run_is_ok_when_the_interactor_spawns_it(self):
        self.assertEqual(self.run_with("print('!', input(), flush=True)", spawn=True), "ok")

    def test_a_rejected_run_is_no_ratio(self):
        self.assertEqual(self.run_with("input(); print('! 8', flush=True)"), "interactor rejects")


class Builds(unittest.TestCase):
    def test_the_packages_stub_links_into_the_accepted_cpp(self):
        stub = bench.package_stub("corpus", "ioi18p1")
        self.assertEqual([os.path.relpath(g, bench.ROOT) for g in stub], ["corpus/problems/ioi18p1/stub.cpp"])
        src = os.path.join(bench.ROOT, "corpus", "submissions", "4707803.cpp")
        exe, err = bench.compile_cpp_cached(src, tempfile.mkdtemp(), "t", extra=stub)
        self.assertIsNone(err)

    def test_children_run_with_an_unlimited_stack(self):
        out = subprocess.run(["sh", "-c", "ulimit -s"], preexec_fn=bench.big_stack, capture_output=True, text=True)
        self.assertEqual(out.stdout.strip(), "unlimited")


if __name__ == "__main__":
    unittest.main()
