#!/usr/bin/env python3
# Enforces handbook/guidelines/conformance-checklist.md#conformancer6-results-agree-across-engines-with-and-without-sanitizers
"""Run the conformance suite several times on one build; report cases whose
verdict changes between runs.

A case that passes on one run and fails on the next hides real regressions
behind noise and makes the scoreboard ratchet lie, so the suite must be
deterministic. Every case runs each time with the same binary.

    uv run scripts/conformance-flakes.py [--runs N] [--check] [filter]
"""
import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONFORM = Path(os.environ.get("CPL_CONFORM", ROOT / "target/debug/cpl-conform"))
FAIL = re.compile(r"^\s+(\S+\.cp): fail")


def failing(filter_: str) -> set[str]:
    r = subprocess.run([str(CONFORM), "conformance", "--verbose", *([filter_] if filter_ else [])],
                       capture_output=True, text=True, cwd=ROOT)
    return {m.group(1) for line in (r.stdout + r.stderr).splitlines() if (m := FAIL.match(line))}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--runs", type=int, default=2)
    ap.add_argument("--check", action="store_true")
    ap.add_argument("filter", nargs="?", default="")
    a = ap.parse_args()
    runs = [failing(a.filter) for _ in range(a.runs)]
    flaky = sorted(set.union(*runs) - set.intersection(*runs))
    for case in flaky:
        print(f"{case}: failed on {sum(case in r for r in runs)} of {a.runs} runs")
    print(f"{len(flaky)} flaky conformance cases over {a.runs} runs")
    return 1 if flaky and a.check else 0


if __name__ == "__main__":
    sys.exit(main())
