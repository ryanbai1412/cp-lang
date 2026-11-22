"""Tests for scripts/land_stats.py, the numbers read from the land log."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import land_stats  # noqa: E402

NOW = 1_000_000


def row(lane, ago, wait, gate=60, holder="h"):
    return {"lane": lane, "landed": NOW - ago, "wait_s": wait, "gate_s": gate,
            "riders": 0, "splits": 0, "holder": holder}


class LandStatsTest(unittest.TestCase):
    def test_wait_percentiles_and_rates_count_only_the_window(self):
        rows = [row("infra", 60, 10), row("site", 60, 30), row("site", 120, 50, holder="s"),
                row("lsp", 2 * 3600, 900)]
        s = land_stats.stats(rows, 1, NOW)
        self.assertEqual((s["landings"], s["queue_wait_p50_s"], s["queue_wait_max_s"]), (3, 30, 50))
        self.assertEqual(s["landings_per_hour_by_lane"], {"infra": 1.0, "site": 2.0})

    def test_riders_of_one_train_count_one_gate(self):
        rows = [row("a", 60, 5, gate=90), row("b", 60, 7, gate=90), row("c", 300, 1, gate=40, holder="c")]
        s = land_stats.stats(rows, 1, NOW)
        self.assertEqual((s["trains"], s["train_gate_max_s"]), (2, 90))

    def test_rider_waits_logged_as_null_are_skipped(self):
        s = land_stats.stats([row("a", 60, None)], 1, NOW)
        self.assertEqual((s["landings"], s["queue_wait_p50_s"]), (1, None))

    def test_the_real_log_parses(self):
        self.assertIsInstance(land_stats.notes(), list)


if __name__ == "__main__":
    unittest.main()
