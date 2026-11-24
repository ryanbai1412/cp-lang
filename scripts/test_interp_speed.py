"""Tests for scripts/interp_speed.py, the land gate on the interpreter's speed."""

import os
import random
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import interp_speed  # noqa: E402

NAMES = [c.name for c in interp_speed.CASES]


def timer(slowdown: float, seed: int = 0):
    """A run's seconds: 3 s per case with up to 4% noise, the tip `slowdown` times slower."""
    rng = random.Random(seed)
    return lambda side, name: 3.0 * (slowdown if side == "tip" else 1.0) * rng.uniform(0.98, 1.02)


class InterpSpeedTest(unittest.TestCase):
    def rows(self, slowdown, seed=0):
        return interp_speed.judge(interp_speed.measure(NAMES, timer(slowdown, seed)))

    def test_a_tip_seeded_1_5x_slower_is_refused(self):
        for seed in range(20):
            rows = self.rows(1.5, seed)
            self.assertTrue(all(r.refused for r in rows), interp_speed.report(rows))
        self.assertIn("REFUSED", interp_speed.report(rows))

    def test_an_unchanged_tip_passes(self):
        # Base and tip draw from one distribution: median of 3 against the base's own spread.
        for seed in range(20):
            rows = self.rows(1.0, seed)
            self.assertFalse([r for r in rows if r.refused], interp_speed.report(rows))

    def test_the_floor_is_the_bases_own_spread_not_a_constant(self):
        times = {"a": ([1.0, 1.1, 1.0, 1.2, 1.0, 1.0], [1.15, 1.15, 1.15]),
                 "b": ([1.0, 1.01, 1.0, 1.0, 1.0, 1.0], [1.15, 1.15, 1.15])}
        a, b = interp_speed.judge(times)
        self.assertAlmostEqual(a.floor, 1.2)
        self.assertFalse(a.refused)
        self.assertAlmostEqual(b.floor, 1.01)
        self.assertTrue(b.refused)
        self.assertIn("noise floor (base vs base, computed): max 1.200", interp_speed.report([a, b]))

    def test_runs_alternate_base_tip_base(self):
        seen = []
        interp_speed.measure(["x"], lambda side, name: seen.append(side) or 1.0, runs=2)
        self.assertEqual(seen, ["base", "tip", "base"] * 2)

    def test_every_case_trims_what_it_reads(self):
        root = interp_speed.ROOT
        for case in interp_speed.CASES:
            self.assertTrue((root / case.program).is_file(), case.program)
            if case.source:
                trimmed = case.trim((root / case.program).read_text())
                self.assertNotEqual(trimmed, (root / case.program).read_text(), case.name)
            if case.max_input:
                self.assertTrue((root / case.program).parent.joinpath("gen.py").is_file(), case.name)


if __name__ == "__main__":
    unittest.main()
