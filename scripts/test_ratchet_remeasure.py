import sys
import types
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import ratchet_remeasure as rr  # noqa: E402


class MeasureError(RuntimeError):
    pass


def mods(hard, fail_until):
    calls = {"n": 0}

    def count(root):
        calls["n"] += 1
        if calls["n"] <= fail_until:
            raise MeasureError("cpl: target/release/cpl is stale: run cargo build --release -p cpl-cli")
        return ["a.cp"]

    m = types.SimpleNamespace(COUNTERS={"c": ("", count, True)}, HARD_ZERO=frozenset({"c"} if hard else ()))
    return {"tip": m, "before": m}


class Measure(unittest.TestCase):
    def test_a_hard_zero_counter_builds_the_debug_cpl_and_measures_again(self):
        built = []
        got = rr.measure(mods(True, 1), "c", ["tip"], Path("."), build=lambda: built.append(1) or True)
        self.assertEqual((got, built), ({"tip": 1}, [1]))

    def test_a_hard_zero_counter_that_still_cannot_measure_refuses(self):
        # 99bc5b2b0: corpus_fmt_dirty passed with only a warning on a stale release cpl.
        with self.assertRaises(rr.Refused):
            rr.measure(mods(True, 9), "c", ["tip"], Path("."), build=lambda: True)

    def test_another_counter_keeps_the_merged_count_with_a_warning(self):
        self.assertIsNone(rr.measure(mods(False, 9), "c", ["tip"], Path("."), build=lambda: False))


if __name__ == "__main__":
    unittest.main()
