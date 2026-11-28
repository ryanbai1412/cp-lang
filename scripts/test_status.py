import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("status", Path(__file__).with_name("status.py"))
status = importlib.util.module_from_spec(spec)
spec.loader.exec_module(status)


class SkippedTableTest(unittest.TestCase):
    def test_counts_trailers_per_lane_per_day(self) -> None:
        log = "\n".join([
            "2026-11-23\x1fperf\x1fbudget, ran over",
            "2026-11-23\x1fperf\x1fa\x1eb",
            "2026-11-23\x1finfra\x1f",
            "2026-11-22\x1f\x1fdocs, late",
        ])
        self.assertEqual(status.skipped_table(log), [
            "| Day | Lane | Skipped |", "|---|---|---|",
            "| 2026-11-23 | perf | 3 |", "| 2026-11-22 | none | 1 |"])

    def test_no_trailers_says_none(self) -> None:
        self.assertEqual(status.skipped_table("2026-11-23\x1finfra\x1f\n"), ["None."])


class WrittenSkippedTableTest(unittest.TestCase):
    def test_a_shallow_clone_rewrites_the_section_unchanged(self) -> None:
        intro = ["Process costing more than 10% of a change is skipped with a trailer",
                 "(`handbook/process/handbook-and-rfcs.md`); this counts them."]
        table = ["| Day | Lane | Skipped |", "|---|---|---|", "| 2026-11-23 | perf | 3 |"]
        text = "\n".join(["# Status", "", status.SKIPPED_HEAD, "", *intro, "", *table, "",
                          "## Open questions (`plan/QUESTIONS.md`)", "", "- q", ""])
        self.assertEqual(status.written_skipped_table(text), table)

    def test_no_section_or_no_table_says_none(self) -> None:
        self.assertEqual(status.written_skipped_table(""), ["None."])
        self.assertEqual(status.written_skipped_table(f"{status.SKIPPED_HEAD}\n\nNone.\n"), ["None."])


class RatchetRisesTest(unittest.TestCase):
    def test_lists_each_number_that_rose(self) -> None:
        old = '{"next_wrong": 998, "std_shadowed": 18, "gone": 1, "ids": ["a"]}'
        new = '{"next_wrong": 1518, "std_shadowed": 17, "added": 4, "ids": ["a", "b"]}'
        self.assertEqual(status.ratchet_rises(old, new), [("next_wrong", 998, 1518)])

    def test_unreadable_text_lists_nothing(self) -> None:
        self.assertEqual(status.ratchet_rises("", '{"a": 1}'), [])
        self.assertEqual(status.ratchet_rises("{", '{"a": 1}'), [])


if __name__ == "__main__":
    unittest.main()
