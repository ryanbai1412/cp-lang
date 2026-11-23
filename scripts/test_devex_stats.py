import importlib.util
import pathlib
import unittest

spec = importlib.util.spec_from_file_location(
    "devex_stats", pathlib.Path(__file__).with_name("devex_stats.py"))
ds = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ds)


class DevexStats(unittest.TestCase):
    def test_devex_commits_count_against_their_lane_and_velocity_bins_by_window(self):
        now = 10_000
        rows = [(now - 60, "lint-fmt", "lint: renumber read_loop to L0564"),
                (now - 120, "lint-fmt", "lint: flag stale heap entry skip"),
                (now - 90, "lint-fmt", "factory: regenerate generated files"),
                (now - 1000, "stdlib", "stdlib: add interval map")]
        s = ds.stats(rows, 0.5, now)
        self.assertEqual(s["lanes"]["lint-fmt"], {"commits": 3, "devex": 1, "share": 0.333})
        self.assertEqual(s["lanes"]["stdlib"]["devex"], 0)
        self.assertEqual(s["velocity_per_15min"], [1, 3])


if __name__ == "__main__":
    unittest.main()
