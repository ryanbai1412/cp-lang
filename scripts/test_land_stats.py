"""Tests for scripts/land_stats.py, the numbers read from the land log."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import land_stats  # noqa: E402

NOW = 1_000_000


def land(domain, ago, seconds):
    return {"domain": domain, "landed": NOW - ago, "seconds": seconds, "tries": 1}


class LandStatsTest(unittest.TestCase):
    def test_duration_percentiles_count_only_the_window(self):
        lands = [land("infra", 60, s) for s in range(1, 21)] + [land("site", 2 * 3600, 900)]
        s = land_stats.stats(lands, [], 1, NOW)
        self.assertEqual((s["landings_logged"], s["land_p50_s"], s["land_p95_s"], s["land_max_s"]),
                         (20, 11, 20, 20))

    def test_commits_per_domain_come_from_the_trailers(self):
        commits = [{"domain": "infra", "landed": NOW - 60}, {"domain": "site", "landed": NOW - 60},
                   {"domain": "site", "landed": NOW - 120}, {"domain": "lsp", "landed": NOW - 7200}]
        s = land_stats.stats([], commits, 1, NOW)
        self.assertEqual(s["commits_per_hour_by_domain"], {"infra": 1.0, "site": 2.0})
        self.assertIsNone(s["land_p95_s"])

    def test_a_trailer_with_a_sentence_after_it_counts_for_its_lane(self):
        self.assertEqual(land_stats.lane("diagnostics. A cut value has no fix."), "diagnostics")
        self.assertEqual(land_stats.lane("lint-fmt (std/strings.cp: one call site)"), "lint-fmt")
        self.assertEqual(land_stats.lane(""), "none")

    def test_the_real_log_and_history_parse(self):
        self.assertIsInstance(land_stats.log_rows(), list)
        self.assertIsInstance(land_stats.trailer_rows(1, "HEAD"), list)


if __name__ == "__main__":
    unittest.main()
