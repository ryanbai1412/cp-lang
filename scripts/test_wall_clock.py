"""A test that asserts a timing budget under 5 s is listed in scripts/ci.py's
WALL_CLOCK_TESTS (or SLOW_TESTS), so no blocking tier runs it (ci/R34, decision 225): a loaded
CI runner must not fail a commit a local run passes. Longer bounds guard hangs."""
import glob
import os
import re
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ci  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BOUND = re.compile(r"(?:from_(millis|secs)\((\d+)\)|as_(millis|secs)\(\)\s*<=?\s*(\d+))")


def listed():
    return set(ci.WALL_CLOCK_TESTS) | set(ci.SLOW_TESTS)


def unlisted_budgets(path, text, allowed):
    out, fn = [], None
    for n, line in enumerate(text.split("\n"), 1):
        if m := re.search(r"\bfn (\w+)\(", line):
            fn = m.group(1)
        if "assert" not in line or "elapsed" not in line:
            continue
        for a, b, c, d in BOUND.findall(line):
            ms = int(b or d) * (1000 if (a or c) == "secs" else 1)
            if ms < 5000 and fn not in allowed:
                out.append(f"{path}:{n} {fn}")
    return out


class WallClock(unittest.TestCase):
    def test_every_short_timing_budget_is_kept_out_of_the_blocking_tiers(self):
        allowed = listed()
        found = []
        for path in sorted(glob.glob(os.path.join(ROOT, "crates", "*", "tests", "*.rs"))):
            if os.path.basename(path) == "budget.rs":
                continue
            with open(path, encoding="utf-8") as f:
                found += unlisted_budgets(os.path.relpath(path, ROOT), f.read(), allowed)
        self.assertEqual(found, [], "add these tests to WALL_CLOCK_TESTS in scripts/ci.py")

    def test_finds_an_unlisted_budget(self):
        text = ("fn quick() {\n    assert!(t.elapsed() < Duration::from_millis(50));\n}\n"
                "fn hang_guard() {\n    assert!(t.elapsed().as_secs() < 10);\n}\n"
                "fn listed_one() {\n    assert!(t.elapsed() < Duration::from_secs(1));\n}\n")
        self.assertEqual(unlisted_budgets("x.rs", text, {"listed_one"}), ["x.rs:2 quick"])
        self.assertIn("ten_million_steps_fit_the_debugging_budgets", listed())


if __name__ == "__main__":
    unittest.main()
