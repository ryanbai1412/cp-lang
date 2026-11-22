#!/usr/bin/env python3
"""Unit tests for scripts/review-check.py.

    python3 -m unittest scripts/test_review_check.py
"""
import importlib.util, os, subprocess, tempfile, unittest
from unittest import mock
from pathlib import Path

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
        errors, _ = rc.evaluate(ports("old", "new"), self.base, {}, set(), {"new"}, touched={"new"})
        self.assertEqual(len(errors), 2)
        errors, _ = rc.evaluate(ports("old", "new"), self.base, {"new": "pending"}, {"new"}, {"new"}, touched={"new"})
        self.assertEqual(errors, [])

    def test_changed_port_needs_review_and_under_4_does_not_block(self):
        # Unrelated change (empty added/touched) is never blocked by under-4 scores or missing reviews.
        reviews = {"a": 3, "b": "pending"}
        all_ports = ports("old", "a", "b")
        errors, st = rc.evaluate(all_ports, self.base, reviews, {"a", "b"}, set(), touched=set())
        self.assertEqual(errors, [])
        self.assertEqual(st["new_under_4"], 1)
        # Adding a new port with dream + review is NOT blocked by other ports scoring under 4.
        errors, _ = rc.evaluate(all_ports, self.base, reviews, {"a", "b"}, {"b"}, touched={"b"})
        self.assertEqual(errors, [])
        # Changing an existing port without a review file fails with an attributable error.
        errors, _ = rc.evaluate(all_ports, self.base, reviews, {"a", "b"}, set(), touched={"old"})
        self.assertEqual(len(errors), 1)
        self.assertIn("old: changed port without a review", errors[0])

    def test_stats(self):
        _, st = rc.evaluate(ports("old", "a"), self.base, {"old": 2, "a": 5}, {"a"}, set())
        self.assertEqual((st["reviewed"], st["median"], st["under_4"]), (2, 3.5, 1))


class AddedPortNamesTest(unittest.TestCase):
    def test_only_the_staged_change_counts(self):
        # A pre-commit hook exports GIT_DIR and GIT_INDEX_FILE; the scratch repo must not see them.
        hook_env = {k: os.environ.pop(k) for k in list(os.environ) if k.startswith("GIT_")}
        self.addCleanup(os.environ.update, hook_env)
        with tempfile.TemporaryDirectory() as d:
            # A commit hook exports GIT_DIR and friends; unset, they would aim these at the real repo.
            env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
            git = lambda *a: subprocess.run(["git", *a], cwd=d, env=env, check=True, capture_output=True)
            git("init", "-q")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "base")
            (Path(d) / "proto/ports/g").mkdir(parents=True)
            (Path(d) / "proto/ports/g/old.cp").write_text("")
            git("add", ".")
            git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "old")
            git("update-ref", "refs/remotes/origin/main", "HEAD~1")
            (Path(d) / "proto/ports/g/new.cp").write_text("")
            git("add", ".")
            root, rc.sb.ROOT = rc.sb.ROOT, Path(d)
            try:
                with mock.patch.dict(os.environ, env, clear=True):
                    self.assertEqual(rc.added_port_names({"old", "new"}), {"new"})
            finally:
                rc.sb.ROOT = root


if __name__ == "__main__":
    unittest.main()
