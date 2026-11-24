"""No conformance case changes verdict between runs of one build.

A case that passes on one run and fails on the next hides real regressions
behind noise and makes the scoreboard lie.
Enforces handbook/guidelines/conformance-checklist.md#conformancer6-results-agree-across-engines-with-and-without-sanitizers
"""
from __future__ import annotations

import os
import re
import subprocess

import specgate
from specgate import Problem

HELP = "every conformance case keeps its verdict over --runs runs of one build"
FAIL = re.compile(r"^\s+(\S+\.cp): fail")


def args(p) -> None:
    p.add_argument("--runs", type=int, default=2)
    p.add_argument("filter", nargs="?", default="")


def failing(filter_: str) -> set[str]:
    conform = os.environ.get("CPL_CONFORM", str(specgate.ROOT / "target/debug/cpl-conform"))
    r = subprocess.run([conform, "conformance", "--verbose", *([filter_] if filter_ else [])],
                       capture_output=True, text=True, cwd=specgate.ROOT)
    return {m.group(1) for line in (r.stdout + r.stderr).splitlines() if (m := FAIL.match(line))}


def run(a) -> list[Problem]:
    runs = [failing(a.filter) for _ in range(a.runs)]
    return [Problem(case, f"failed on {sum(case in r for r in runs)} of {a.runs} runs")
            for case in sorted(set.union(*runs) - set.intersection(*runs))]
