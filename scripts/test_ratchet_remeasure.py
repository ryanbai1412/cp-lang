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


class MeasureAllTest(unittest.TestCase):
    def test_each_job_is_measured_in_order_and_a_refusal_comes_back(self):
        side = types.SimpleNamespace(COUNTERS={"a": ("", lambda r: [1], True), "b": ("", lambda r: [1, 2], True),
                                               "z": ("", lambda r: 1 / 0, True)}, HARD_ZERO={"z"})
        m = {"tip": side, "before": side}
        jobs = [("a", ["before"], ["before"]), ("b", ["before"], ["before"]), ("z", ["before"], ["before"])]
        got = rr.measure_all(m, jobs, Path("."))
        self.assertEqual(got[:2], [{"tip": 1, "before": 1}, {"tip": 2, "before": 2}])
        self.assertIsInstance(got[2], rr.Refused)


class RestateTest(unittest.TestCase):
    def test_a_rule_change_moves_a_baseline_by_what_it_stops_counting(self):
        self.assertEqual(rr.restate(147, 147, 139), 139)

    def test_debt_over_the_baseline_is_not_taken_off_it(self):
        self.assertEqual(rr.restate(147, 155, 147), 147)
        self.assertEqual(rr.restate(147, 155, 150), 147)
        self.assertEqual(rr.restate(147, 155, 140), 140)


    def test_a_commit_bringing_the_rule_takes_the_tips_measure(self):
        counts = {"before": {"c": 597}, "upstream": {"c": 59}}
        sides = ["before", "upstream"]
        self.assertEqual(rr.merged(counts, "c", sides, ["upstream"], {"upstream": 59, "tip": 603}), 603)
        self.assertEqual(rr.merged(counts, "c", sides, ["upstream"], {"upstream": 59, "tip": 590}), 590)

    def test_a_rule_main_changed_restates_this_commits_number(self):
        counts = {"before": {"c": 150}, "upstream": {"c": 147}}
        got = rr.merged(counts, "c", ["before", "upstream"], ["before"], {"before": 155, "tip": 147})
        self.assertEqual(got, 147)

    def test_a_hard_zero_counter_at_zero_everywhere_is_not_remeasured(self):
        tip = types.SimpleNamespace(HARD_ZERO={"z"})
        counts = {"before": {"z": 0, "n": 0}, "upstream": {"z": 0, "n": 0}}
        self.assertTrue(rr.settled({"tip": tip}, counts, "z", ["before", "upstream"]))
        self.assertFalse(rr.settled({"tip": tip}, counts, "n", ["before", "upstream"]))
        self.assertFalse(rr.settled({"tip": tip}, {**counts, "before": {"z": 1}}, "z", ["before", "upstream"]))


if __name__ == "__main__":
    unittest.main()
