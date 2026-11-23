"""Tests for scripts/coverage-matrix.py."""

import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("cm", Path(__file__).with_name("coverage-matrix.py"))
cm = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cm)


def entry(path: str, covered: int, count: int) -> dict:
    return {"filename": str(cm.ROOT / path),
            "summary": {k: {"covered": covered, "count": count} for k in cm.KINDS}}


class CoverageMatrixTest(unittest.TestCase):
    def test_per_crate_sums_only_each_crates_src_files(self) -> None:
        report = {"data": [{"files": [
            entry("crates/a/src/lib.rs", 3, 4), entry("crates/a/src/x/y.rs", 1, 4),
            entry("crates/a/tests/t.rs", 9, 9), entry("crates/b/src/lib.rs", 0, 2),
            entry("crates/c/src/lib.rs", 5, 5), {"filename": "/elsewhere/lib.rs", "summary": {}}]}]}
        got = cm.per_crate(report, ["a", "b"])
        self.assertEqual(got["a"]["lines"], [4, 8])
        self.assertEqual(got["b"]["functions"], [0, 2])
        self.assertNotIn("c", got)

    def test_table_shows_own_and_all_columns(self) -> None:
        own = {"a": {k: [1, 2] for k in cm.KINDS}}
        all_ = {"a": {k: [2, 2] for k in cm.KINDS}, "b": {k: [0, 0] for k in cm.KINDS}}
        lines = cm.table(own, all_).splitlines()
        self.assertEqual(lines[2], "| a | 50% | 50% | 50% | 100% | 100% | 100% |")
        self.assertEqual(lines[3], "| b |  |  |  | - | - | - |")


if __name__ == "__main__":
    unittest.main()
