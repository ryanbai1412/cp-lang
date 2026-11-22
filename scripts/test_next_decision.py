#!/usr/bin/env python3
"""Unit tests for scripts/next-decision.py."""
import importlib.util
import os
import unittest

SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "next-decision.py")
SPEC = importlib.util.spec_from_file_location("next_decision", SCRIPT)
assert SPEC and SPEC.loader
MOD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MOD)


class NextDecisionTest(unittest.TestCase):
    def test_empty_starts_at_one(self):
        self.assertEqual(MOD.next_number("# Decisions\n"), 1)

    def test_next_number_is_max_plus_one(self):
        text = "## 1. First\nbody\n## 42. Later\n### 99. Not a decision\n## 10. Earlier\n"
        self.assertEqual(MOD.parse_decision_numbers(text), [1, 42, 10])
        self.assertEqual(MOD.next_number(text), 43)
        self.assertEqual(MOD.find_duplicates(text), [])

    def test_finds_duplicates(self):
        text = "## 5. One\n## 6. Two\n## 5. Colliding\n"
        self.assertEqual(MOD.find_duplicates(text), [5])


if __name__ == "__main__":
    unittest.main()
