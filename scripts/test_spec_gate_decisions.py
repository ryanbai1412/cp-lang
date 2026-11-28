#!/usr/bin/env python3
"""Unit tests for `scripts/spec-gate.py decisions`."""
import contextlib
import io
import os
import sys
import subprocess
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from specgate import decisions as MOD  # noqa: E402


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
        text = MOD.with_stub("## 4. A\nbody\n", 5, "content", "a title")
        self.assertEqual(MOD.parse_decision_numbers(text), [4, 5])
        self.assertTrue(MOD.STUB_RE.search(text))


def _git(cwd, *args):
    return subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True, text=True).stdout


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
        self.assertEqual(MOD.reserve("one", "first", "spec", "origin", 3), 0)
        self._use(b)
        self.assertEqual(MOD.reserve("two", "second", "spec", "origin", 3), 0)
        _git(b, "fetch", "-q", "origin", "main")
        main = MOD.show("FETCH_HEAD")
        self.assertEqual(MOD.parse_decision_numbers(main), [1, 2, 3])
        self.assertEqual(len(MOD.STUB_RE.findall(main)), 2)
        _git(b, "pull", "-q", "--rebase", "origin", "main")
        path = os.path.join(b, "decisions.md")
        text = open(path).read().replace("## 3. (reserved, two: second)", "## 3. Taken")
        open(path, "w").write(text)
        _git(b, "commit", "-qam", "use 3")
        _git(b, "fetch", "-q", "origin", "main")
        self.assertEqual(MOD.check_landing("FETCH_HEAD"), 0)
        open(path, "a").write("\n## 4. Unreserved\n")
        _git(b, "commit", "-qam", "take 4")
        self.assertEqual(MOD.check_landing("FETCH_HEAD"), 1)
        self.assertEqual(MOD.check_landing("no-such-ref"), 2)

    def test_a_reservation_names_its_lane_and_title_or_refuses(self):
        a = self.clones[0]
        self._use(a)
        before = _git(a, "ls-remote", "origin", "main")
        for lane, title in [(None, "views"), ("unknown", "views"), ("E", None), ("E", "  "), ("E", "a (b)")]:
            self.assertEqual(MOD.reserve(lane, title, "spec", "origin", 3), 2, (lane, title))
        self.assertEqual(_git(a, "ls-remote", "origin", "main"), before)
        self.assertEqual(MOD.reserve("E", "views own their keys", "spec", "origin", 3), 0)
        _git(a, "fetch", "-q", "origin", "main")
        self.assertIn("## 2. (reserved, E: views own their keys)", MOD.show("FETCH_HEAD"))
        self.assertEqual(len(MOD.STUB_RE.findall(MOD.show("FETCH_HEAD"))), 1)
        body = _git(a, "log", "-1", "--format=%B", "FETCH_HEAD")
        self.assertIn("views own their keys", body)
        self.assertEqual(_git(a, "log", "-1", "--format=%(trailers:key=Lane,valueonly)", "FETCH_HEAD").strip(), "E")

    def _hook(self, hooks, name, body):
        os.makedirs(hooks, exist_ok=True)
        path = os.path.join(hooks, name)
        with open(path, "w") as fh:
            fh.write("#!/bin/sh\n" + body)
        os.chmod(path, 0o755)

    def test_reserve_skips_the_local_pre_push_hook(self):
        a = self.clones[0]
        hooks = os.path.join(self.tmp.name, "hooks")
        self._hook(hooks, "pre-push", "exit 1\n")
        _git(a, "config", "core.hooksPath", hooks)
        self._use(a)
        self.assertEqual(MOD.reserve("one", "first", "spec", "origin", 3), 0)

    def test_a_refused_push_says_why_and_does_not_retry(self):
        self._hook(os.path.join(self.remote, "hooks"), "pre-receive", "echo branch is protected >&2; exit 1\n")
        self._use(self.clones[0])
        err = io.StringIO()
        with contextlib.redirect_stderr(err):
            self.assertEqual(MOD.reserve("one", "first", "spec", "origin", 3), 1)
        self.assertIn("branch is protected", err.getvalue())
        self.assertNotIn("retry:", err.getvalue())


if __name__ == "__main__":
    unittest.main()
