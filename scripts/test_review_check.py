#!/usr/bin/env python3
"""Unit tests for scripts/review-check.py.

    python3 -m unittest scripts/test_review_check.py
"""
import importlib.util, os, unittest

_spec = importlib.util.spec_from_file_location(
    "review_check", os.path.join(os.path.dirname(os.path.abspath(__file__)), "review-check.py"))
rc = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(rc)


def ports(*names):
    return [{"name": n} for n in names]


class ParseScoreTest(unittest.TestCase):
    def test_reads_score_pending_and_missing(self):
        self.assertEqual(rc.parse_score("# x\n\nScore: 4/5\n"), 4)
        self.assertEqual(rc.parse_score("Score: pending\n"), "pending")
        self.assertIsNone(rc.parse_score("no score here, Score: 4/5 mid-line"))

    def test_reads_dream_score(self):
        self.assertEqual(rc.parse_dream_score("Score: 4/5\nDream: 3/5\n"), 3)
        self.assertIsNone(rc.parse_dream_score("Score: 4/5\n"))


class EvaluateTest(unittest.TestCase):
    base = {"ports": ["old"], "new_under_4_max": 0}

    def test_old_ports_need_nothing(self):
        errors, st = rc.evaluate(ports("old"), self.base, {}, set(), set())
        self.assertEqual(errors, [])
        self.assertEqual(st["new"], 0)

    def test_new_port_needs_dream_and_review(self):
        errors, _ = rc.evaluate(ports("old", "new"), self.base, {}, set(), set())
        self.assertEqual(len(errors), 2)
        errors, _ = rc.evaluate(ports("old", "new"), self.base, {"new": "pending"}, {"new"}, set())
        self.assertEqual(errors, [])

    def test_ratchet_blocks_only_a_change_adding_a_port(self):
        reviews = {"a": 3, "b": "pending"}
        all_ports = ports("old", "a", "b")
        errors, st = rc.evaluate(all_ports, self.base, reviews, {"a", "b"}, set())
        self.assertEqual(errors, [])
        self.assertEqual(st["new_under_4"], 1)
        errors, _ = rc.evaluate(all_ports, self.base, reviews, {"a", "b"}, {"b"})
        self.assertTrue(errors and errors[0].startswith("ratchet"))
        errors, _ = rc.evaluate(all_ports, dict(self.base, new_under_4_max=1), reviews, {"a", "b"}, {"b"})
        self.assertEqual(errors, [])

    def test_stats(self):
        _, st = rc.evaluate(ports("old", "a"), self.base, {"old": 2, "a": 5}, {"a"}, set())
        self.assertEqual((st["reviewed"], st["median"], st["under_4"]), (2, 3.5, 1))


if __name__ == "__main__":
    unittest.main()
