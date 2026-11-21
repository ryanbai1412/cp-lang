#!/usr/bin/env python3
"""Unit tests for scripts/check_cache.py and .githooks/pre-push."""

import os
import subprocess
import tempfile
import unittest

import scripts.check_cache as cc


class CheckCacheTest(unittest.TestCase):
    def test_hash_changes_on_edit_and_ignores_pycache(self):
        with tempfile.TemporaryDirectory() as tmp:
            os.makedirs(os.path.join(tmp, "src", "__pycache__"), exist_ok=True)
            f1 = os.path.join(tmp, "src", "a.rs")
            with open(f1, "w", encoding="utf-8") as f:
                f.write("fn main() {}\n")
            d1 = cc.hash_paths(tmp, ["src"])

            # Writing inside __pycache__ must not invalidate the hash.
            with open(os.path.join(tmp, "src", "__pycache__", "x.pyc"), "wb") as f:
                f.write(b"1234")
            self.assertEqual(cc.hash_paths(tmp, ["src"]), d1)

            # Editing a tracked source file must invalidate the hash.
            with open(f1, "w", encoding="utf-8") as f:
                f.write("fn main() { let _ = 1; }\n")
            d2 = cc.hash_paths(tmp, ["src"])
            self.assertNotEqual(d1, d2)

            cache_dir = os.path.join(tmp, "cache")
            self.assertFalse(cc.is_hit(cache_dir, "clippy", d2))
            cc.save_hit(cache_dir, "clippy", d2)
            self.assertTrue(cc.is_hit(cache_dir, "clippy", d2))
            self.assertFalse(cc.is_hit(cache_dir, "clippy", d1))

    def test_pre_push_hook_rejects_conflict_markers(self):
        root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        hook = os.path.join(root, ".githooks", "pre-push")
        self.assertTrue(os.path.isfile(hook))
        self.assertTrue(os.access(hook, os.X_OK))
        with tempfile.TemporaryDirectory() as tmp:
            subprocess.run(["git", "init", "-q"], cwd=tmp, check=True)
            subprocess.run(["git", "config", "user.email", "t@example.com"], cwd=tmp, check=True)
            subprocess.run(["git", "config", "user.name", "t"], cwd=tmp, check=True)
            p = os.path.join(tmp, "bad.md")
            with open(p, "w", encoding="utf-8") as f:
                f.write("hello\n<<<<<<< HEAD\nfoo\n=======\nbar\n>>>>>>> other\n")
            subprocess.run(["git", "add", "bad.md"], cwd=tmp, check=True)
            r = subprocess.run([hook], cwd=tmp, capture_output=True, text=True)
            self.assertNotEqual(r.returncode, 0)
            self.assertIn("merge conflict markers", r.stderr)


if __name__ == "__main__":
    unittest.main()
