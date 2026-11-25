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


class RetargetedTest(unittest.TestCase):
    OLD = [{"id": 1, "name": "a", "size": "6,000 cases", "missing": "the rest"},
           {"id": 2, "name": "b", "size": "all"}]

    def test_a_weakened_size_needs_a_changed_note(self) -> None:
        new = [dict(self.OLD[0], size="300 cases"), self.OLD[1]]
        self.assertEqual(len(end_state.retargeted(self.OLD, new)), 1)
        new[0]["changed"] = "2026-11-25: the suite was split"
        self.assertEqual(end_state.retargeted(self.OLD, new), [])

    def test_an_old_note_does_not_cover_a_new_change(self) -> None:
        old = [dict(self.OLD[0], changed="2026-11-25: first")]
        new = [dict(old[0], missing="")]
        self.assertEqual(len(end_state.retargeted(old, new)), 1)

    def test_a_dropped_suite_is_reported_and_a_new_one_is_not(self) -> None:
        self.assertEqual(len(end_state.retargeted(self.OLD, self.OLD[:1])), 1)
        self.assertEqual(end_state.retargeted(self.OLD, self.OLD + [{"id": 3, "name": "c"}]), [])


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
