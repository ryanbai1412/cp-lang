#!/usr/bin/env python3
"""Unit tests for scripts/next-decision.py."""
import importlib.util
import os
import subprocess
import tempfile
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

    def test_unreserved_is_a_number_the_base_never_had(self):
        base = "## 1. A\n## 2. (reserved, content)\n"
        self.assertEqual(MOD.unreserved(base, base.replace("(reserved, content)", "B")), [])
        self.assertEqual(MOD.unreserved(base, base + "## 3. C\n"), [3])

    def test_stub_is_appended_as_a_heading(self):
        text = MOD.with_stub("## 4. A\nbody\n", 5, "content")
        self.assertEqual(MOD.parse_decision_numbers(text), [4, 5])
        self.assertTrue(MOD.STUB_RE.search(text))


def _git(cwd, *args):
    subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True, text=True)


class ReserveTest(unittest.TestCase):
    """Two clones reserving in turn get distinct numbers, and a landing checks."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        t = self.tmp.name
        self.remote = os.path.join(t, "remote.git")
        _git(t, "init", "-q", "--bare", "-b", "main", self.remote)
        self.clones = []
        for name in ("a", "b"):
            path = os.path.join(t, name)
            _git(t, "clone", "-q", self.remote, path)
            _git(path, "config", "user.email", "t@example.com")
            _git(path, "config", "user.name", "t")
            _git(path, "checkout", "-q", "-b", "main")
            self.clones.append(path)
        a = self.clones[0]
        with open(os.path.join(a, "decisions.md"), "w") as fh:
            fh.write("# Decisions\n\n## 1. First\n")
        _git(a, "add", "decisions.md")
        _git(a, "commit", "-qm", "seed")
        _git(a, "push", "-q", "origin", "main")
        self.saved = MOD.ROOT, MOD.DECISIONS

    def tearDown(self):
        MOD.ROOT, MOD.DECISIONS = self.saved
        self.tmp.cleanup()

    def _use(self, path):
        MOD.ROOT, MOD.DECISIONS = path, os.path.join(path, "decisions.md")

    def test_reserve_then_land(self):
        a, b = self.clones
        self._use(a)
        self.assertEqual(MOD.reserve("one", "spec", "origin", 3), 0)
        self._use(b)
        self.assertEqual(MOD.reserve("two", "spec", "origin", 3), 0)
        _git(b, "fetch", "-q", "origin", "main")
        main = MOD.show("FETCH_HEAD")
        self.assertEqual(MOD.parse_decision_numbers(main), [1, 2, 3])
        self.assertEqual(len(MOD.STUB_RE.findall(main)), 2)
        _git(b, "pull", "-q", "--rebase", "origin", "main")
        path = os.path.join(b, "decisions.md")
        text = open(path).read().replace("## 3. (reserved, two)", "## 3. Taken")
        open(path, "w").write(text)
        _git(b, "commit", "-qam", "use 3")
        _git(b, "fetch", "-q", "origin", "main")
        self.assertEqual(MOD.check_landing("FETCH_HEAD"), 0)
        open(path, "a").write("\n## 4. Unreserved\n")
        _git(b, "commit", "-qam", "take 4")
        self.assertEqual(MOD.check_landing("FETCH_HEAD"), 1)
        self.assertEqual(MOD.check_landing("no-such-ref"), 2)


if __name__ == "__main__":
    unittest.main()
