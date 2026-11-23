#!/usr/bin/env python3
"""List landed commits that change expected output together with code.

A commit that edits a saved expectation (`.out`, `.err`, `.trace`, a
golden) in the same change as the code that produces it may have blessed a
bug. Adding a new expectation file is fine; changing an existing one next
to code is flagged for post-land review.

Usage:
    python3 scripts/expectation-edits.py [--since "1 hour ago"] [REV_RANGE]
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EXPECTATION_RE = re.compile(r"\.(out|err|trace|stderr|expected|golden|snap)$")
CODE_RE = re.compile(r"^(crates/[^/]+/src/.+\.rs|std/.+\.cp)$")
TEST_CODE_RE = re.compile(r"(^|/)(tests?|[a-z_]*_tests?)\.rs$|/tests/")


def flagged(changes: list[tuple[str, str]]) -> tuple[list[str], list[str]] | None:
    """The modified expectations and the code beside them, if both are present."""
    expectations = [p for s, p in changes if s.startswith("M") and EXPECTATION_RE.search(p)]
    code = [p for _, p in changes if CODE_RE.match(p) and not TEST_CODE_RE.search(p)]
    return (expectations, code) if expectations and code else None


def commits(root: Path, args: list[str]) -> list[tuple[str, str, list[tuple[str, str]]]]:
    out = subprocess.run(["git", "log", "--no-merges", "--name-status", "--format=@@%h %s", *args],
                         cwd=root, capture_output=True, text=True, check=True).stdout
    found, cur = [], None
    for line in out.splitlines():
        if line.startswith("@@"):
            sha, _, subject = line[2:].partition(" ")
            cur = (sha, subject, [])
            found.append(cur)
        elif line.strip() and cur is not None:
            status, *paths = line.split("\t")
            cur[2].append((status, paths[-1]))
    return found


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--since", default=None, help='e.g. "1 hour ago"')
    parser.add_argument("range", nargs="?", default=None, help="git revision range, e.g. A..B")
    args = parser.parse_args(argv)
    log_args = ([f"--since={args.since}"] if args.since else []) + ([args.range] if args.range else [])
    hits = 0
    for sha, subject, changes in commits(ROOT, log_args or ["--since=1 hour ago"]):
        if f := flagged(changes):
            hits += 1
            print(f"{sha} {subject}\n  {len(f[0])} expectation(s): {', '.join(f[0][:5])}"
                  f"{' ...' if len(f[0]) > 5 else ''}\n  code: {', '.join(f[1][:5])}{' ...' if len(f[1]) > 5 else ''}")
    print(f"expectation-edits: {hits} commit(s) to review")
    return 0


if __name__ == "__main__":
    sys.exit(main())
