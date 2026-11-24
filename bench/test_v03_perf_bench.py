import importlib.util
import os
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


if __name__ == "__main__":
    unittest.main()
