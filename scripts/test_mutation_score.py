import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("ms", Path(__file__).with_name("mutation-score.py"))
ms = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ms)


class ScoreTest(unittest.TestCase):
    def test_score_ignores_unviable_and_timeouts(self) -> None:
        s = ms.score({"caught": 3, "missed": 1, "timeout": 2, "unviable": 5})
        self.assertEqual((s["score"], s["timeout"], s["unviable"]), (0.75, 2, 5))
        self.assertIsNone(ms.score({"unviable": 2})["score"])

    def test_missed_names_lists_only_missed_mutants(self) -> None:
        outcomes = {"outcomes": [
            {"scenario": "Baseline", "summary": "Success"},
            {"scenario": {"Mutant": {"name": "a.rs:1: replace f -> bool with true"}}, "summary": "MissedMutant"},
            {"scenario": {"Mutant": {"name": "a.rs:2: delete !"}}, "summary": "CaughtMutant"},
        ]}
        self.assertEqual(ms.missed_names(outcomes), ["a.rs:1: replace f -> bool with true"])

    def test_min_fails_only_a_score_below_it(self) -> None:
        self.assertEqual(ms.below({"score": 0.98}, 0.98), 0)
        self.assertEqual(ms.below({"score": 0.97}, 0.98), 1)
        self.assertEqual(ms.below({"score": None}, 0.98), 1)
        self.assertEqual(ms.below({"score": 0.1}, None), 0)

    def test_runs_the_baseline_so_a_red_tree_scores_nothing(self) -> None:
        cmd = ms.mutants_command("cpl-fmt", Path("o"), "4", ["cpl-cli"])
        self.assertNotIn("--baseline", cmd)
        self.assertEqual(cmd[-2:], ["--test-package", "cpl-fmt,cpl-cli"])


if __name__ == "__main__":
    unittest.main()
