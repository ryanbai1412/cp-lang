import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("end_state", Path(__file__).with_name("end-state.py"))
end_state = importlib.util.module_from_spec(spec)
spec.loader.exec_module(end_state)


class StateTest(unittest.TestCase):
    def test_done_needs_every_number_at_its_target(self) -> None:
        suite = {"numbers": [{"counter": "a", "target": 0}, {"counter": "b", "target": 0}]}
        self.assertEqual(end_state.state(suite, {"a": 0, "b": 0})[0], "done")
        self.assertEqual(end_state.state(suite, {"a": 0, "b": 1_200}), ("measured", "`a` 0 → 0, `b` 1,200 → 0"))

    def test_a_missing_number_keeps_a_suite_unmeasured_even_at_zero(self) -> None:
        suite = {"numbers": [{"counter": "a", "target": 0}], "missing": "the rest"}
        self.assertEqual(end_state.state(suite, {"a": 0}), ("unmeasured", "`a` 0 → 0; no number yet for: the rest"))


class CommittedFileTest(unittest.TestCase):
    def test_the_committed_suites_are_well_formed(self) -> None:
        suites, _, errors = end_state.load()
        self.assertEqual(errors, [])
        self.assertGreaterEqual(len(suites), end_state.SUITES_AT_LEAST)

    def test_every_suite_has_a_row_and_a_section(self) -> None:
        suites, counts, _ = end_state.load()
        text = end_state.render(suites, counts)
        for s in suites:
            self.assertIn(f"| {s['id']} | {s['name']} |", text)
            self.assertIn(f"### {s['id']}. {s['name']}", text)


if __name__ == "__main__":
    unittest.main()
