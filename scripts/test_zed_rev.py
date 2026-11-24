"""scripts/zed-rev.py: the Zed grammar rev stays at a pushed commit holding the tree's grammar."""

from __future__ import annotations

import importlib.util
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("zed_rev", Path(__file__).parent / "zed-rev.py")
zed_rev = importlib.util.module_from_spec(spec)
spec.loader.exec_module(zed_rev)

GRAMMAR = "editors/tree-sitter-cp/grammar.js"
ENV = {"GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.com",
       "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@example.com"}


def git(cwd: Path, *args: str) -> str:
    return subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True, text=True,
                          env={**os.environ, **ENV}).stdout.strip()


def commit(clone: Path, grammar: str, message: str) -> str:
    (clone / GRAMMAR).write_text(grammar)
    git(clone, "commit", "-qam", message)
    git(clone, "push", "-q", "origin", "HEAD:main")
    return git(clone, "rev-parse", "HEAD")


class ZedRev(unittest.TestCase):
    def setUp(self) -> None:
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.tmp = Path(tmp.name)
        origin = self.tmp / "origin.git"
        git(self.tmp, "init", "-q", "--bare", "-b", "main", str(origin))
        seed = self.tmp / "seed"
        git(self.tmp, "clone", "-q", str(origin), str(seed))
        for file in zed_rev.FILES:
            (seed / file).parent.mkdir(parents=True, exist_ok=True)
            (seed / file).write_text("one\n")
        (seed / zed_rev.TOML).parent.mkdir(parents=True)
        (seed / zed_rev.TOML).write_text('[grammars.cp]\nrev = "0000000000000000000000000000000000000000"\n')
        git(seed, "add", "-A")
        git(seed, "commit", "-qm", "seed")
        git(seed, "push", "-q", "origin", "HEAD:main")
        self.first = git(seed, "rev-parse", "HEAD")
        self.seed = seed
        self.clone = self.tmp / "regen"
        git(self.tmp, "clone", "-q", str(origin), str(self.clone))
        old = zed_rev.ROOT
        zed_rev.ROOT = self.clone
        self.addCleanup(setattr, zed_rev, "ROOT", old)

    def rev(self) -> str:
        return zed_rev.REV.search((self.clone / zed_rev.TOML).read_text())[1]

    def test_a_rev_without_the_trees_grammar_is_stale_and_rewritten(self) -> None:
        self.assertEqual(zed_rev.main(["--check"]), 1)
        self.assertEqual(zed_rev.main(["--write"]), 0)
        self.assertEqual(self.rev(), self.first)
        self.assertEqual(zed_rev.main(["--check"]), 0)

    def test_a_grammar_commit_rebased_under_the_written_rev_is_written_again(self) -> None:
        zed_rev.main(["--write"])
        git(self.clone, "commit", "-qam", "regen")
        newer = commit(self.seed, "two\n", "grammar")
        git(self.clone, "pull", "-q", "--rebase", "origin", "main")
        self.assertEqual(self.rev(), self.first)
        self.assertEqual(zed_rev.main(["--check"]), 1)
        self.assertEqual(zed_rev.main(["--write"]), 0)
        self.assertEqual(self.rev(), newer)

    def test_a_grammar_not_yet_on_main_is_not_written(self) -> None:
        zed_rev.main(["--write"])
        (self.clone / GRAMMAR).write_text("local\n")
        self.assertEqual(zed_rev.main(["--write"]), 1)
        self.assertEqual(self.rev(), self.first)


if __name__ == "__main__":
    unittest.main()
