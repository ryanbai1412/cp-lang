#!/usr/bin/env python3
"""Keep editors/zed/extension.toml's grammar `rev` at a pushed commit whose
editors/tree-sitter-cp is the one in the tree (Zed fetches the grammar there).

    scripts/zed-rev.py --check   fail when the rev's grammar differs from the tree
    scripts/zed-rev.py --write   point the rev at the newest such commit on origin/main

A commit can't name itself, so the regen job (gen.toml `zed-rev`) writes the
rev after a grammar change lands, and again after its own rebase: a grammar
commit landed meanwhile makes the rev it wrote stale.
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TOML = "editors/zed/extension.toml"
FILES = ["editors/tree-sitter-cp/grammar.js", "editors/tree-sitter-cp/src/parser.c"]
REV = re.compile(r'^rev = "([0-9a-f]+)"$', re.M)


def git(*args):
    return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True)


def has_tree_grammar(rev):
    """Whether `rev` (fetched if this clone lacks it) holds the tree's grammar."""
    if git("cat-file", "-e", f"{rev}^{{commit}}").returncode != 0:
        git("fetch", "-q", "--depth", "1", "origin", rev)
    for file in FILES:
        shown = git("show", f"{rev}:{file}")
        if shown.returncode != 0 or shown.stdout != (ROOT / file).read_bytes():
            return False
    return True


def newest():
    """The newest commit on origin/main with the tree's grammar: the last one to
    change it, else (a shallow clone) the tip."""
    git("fetch", "-q", "origin", "main")
    last = git("log", "-1", "--format=%H", "origin/main", "--", *FILES).stdout.decode().strip()
    tip = git("rev-parse", "origin/main").stdout.decode().strip()
    return next((rev for rev in (last, tip) if rev and has_tree_grammar(rev)), None)


def main(argv):
    toml = ROOT / TOML
    text = toml.read_text()
    found = REV.search(text)
    if not found:
        print(f"zed-rev: no `rev = \"...\"` in {TOML}", file=sys.stderr)
        return 1
    if has_tree_grammar(found[1]):
        return 0
    if "--write" not in argv:
        print(f"zed-rev: editors/tree-sitter-cp changed since rev {found[1]}; run scripts/zed-rev.py --write",
              file=sys.stderr)
        return 1
    rev = newest()
    if rev is None:
        print("zed-rev: no commit on origin/main has the tree's grammar yet; land it first", file=sys.stderr)
        return 1
    toml.write_text(text[: found.start(1)] + rev + text[found.end(1):])
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
