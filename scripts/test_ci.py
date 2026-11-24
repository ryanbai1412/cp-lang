import subprocess
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import ci  # noqa: E402


class GatePartTest(unittest.TestCase):
    def test_checks_and_unset(self) -> None:
        for part in ("", "checks"):
            self.assertEqual(ci.gate_part(part, "commit")["kind"], "checks")

    def test_conform_suites_and_shard(self) -> None:
        self.assertEqual(ci.gate_part("conform", "fast"),
                         {"kind": "conform", "suites": "all", "shard": "", "partition": ""})
        g = ci.gate_part("conform:conformance+lints,std@2/3", "fast")
        self.assertEqual((g["suites"], g["shard"]), ("conformance lints std", "--shard 2/3"))

    def test_tests_partitions(self) -> None:
        self.assertEqual(ci.gate_part("tests:doc", "fast")["kind"], "doc")
        g = ci.gate_part("tests:hash:1/12", "fast")
        self.assertEqual((g["kind"], g["partition"]), ("tests", "hash:1/12"))

    def test_parts_need_fast(self) -> None:
        for part in ("fmt", "conform", "conform:all", "tests:doc", "tests:hash:1/2"):
            with self.assertRaisesRegex(ValueError, "needs --fast"):
                ci.gate_part(part, "land")

    def test_unknown_and_malformed(self) -> None:
        for part in ("bogus", "tests:", "conform:all@"):
            with self.assertRaises(ValueError):
                ci.gate_part(part, "fast")


class FilterTest(unittest.TestCase):
    def test_blocking_tiers_skip_every_wall_clock_test(self) -> None:
        slow, wall = ci.slow_filter(), ci.wall_filter()
        for name in ci.WALL_CLOCK_TESTS:
            clause = f"test(/(^|::){name}$/)"
            self.assertIn(clause, slow)
            self.assertIn(clause, wall)
        for name in ci.SLOW_TESTS:
            self.assertIn(f"test(/(^|::){name}$/)", slow)
        self.assertTrue(slow.startswith("binary(budget)"))

    def test_plan_round_trips_through_the_shell(self) -> None:
        out = subprocess.run(
            ["bash", "-c", 'eval "$(python3 scripts/ci.py plan --mode fast --gate-part conform:a+b@1/2)"; '
             'printf "%s|%s|%s\\n" "$gate_kind" "$conform_suites" "$conform_shard"; printf "%s" "$slow_filter"'],
            cwd=Path(__file__).parent.parent, capture_output=True, text=True, check=True).stdout
        first, slow = out.split("\n", 1)
        self.assertEqual(first, "conform|a b|--shard 1/2")
        self.assertEqual(slow, ci.slow_filter())

    def test_plan_rejects_a_bad_part(self) -> None:
        self.assertEqual(ci.main(["plan", "--mode", "fast", "--gate-part", "bogus"]), 1)


if __name__ == "__main__":
    unittest.main()
