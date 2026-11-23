#!/usr/bin/env python3
# Enforces handbook/guidelines/conformance-checklist.md#conformancer3-a-known-gap-fails-for-the-reason-it-names
"""Report how long each known-gap conformance case has been failing, by owner.

A known gap is a case research/scoreboard.json records as failing. Its age
is the days since the case file was added. Cases under `core-*` and
`diagnostics` belong to the compiler, `std-*` and `lib-*` to stdlib, and
`lints` to lint; the report gives each owner's count, median and oldest
age so a gap that is listed but never fixed is visible.

    uv run scripts/conformance-gap-age.py [--oldest N] [--max-days D]

With `--max-days`, exit 1 if any gap is older than D days.
"""
import argparse
import json
import statistics
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OWNERS = (("core-", "compiler"), ("diagnostics", "compiler"), ("std-", "stdlib"),
          ("lib-", "stdlib"), ("lints", "lint"))


def owner(case: str) -> str:
    return next((o for p, o in OWNERS if case.startswith(p)), "spec")


def added(root: Path) -> dict[str, int]:
    """Case path -> unix time of the commit that first added it."""
    out = subprocess.run(["git", "log", "--diff-filter=A", "--format=@%ct", "--name-only",
                          "--", "spec/conformance"], cwd=root, capture_output=True, text=True,
                         check=True).stdout
    first, t = {}, 0
    for line in out.splitlines():
        if line.startswith("@"):
            t = int(line[1:])
        elif line.endswith(".cp"):
            first[line.removeprefix("spec/conformance/")] = t
    return first


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--oldest", type=int, default=10)
    ap.add_argument("--max-days", type=float)
    a = ap.parse_args()
    cases = json.loads((ROOT / "research/scoreboard.json").read_text())["suites"]["conformance"]["cases"]
    when, now = added(ROOT), time.time()
    ages = {c: (now - when.get(c, now)) / 86400 for c, v in cases.items() if v != "pass"}
    by = {}
    for c, d in ages.items():
        by.setdefault(owner(c), []).append(d)
    for o, ds in sorted(by.items()):
        print(f"{o:9} {len(ds):4} gaps, median {statistics.median(ds):5.1f} days, oldest {max(ds):5.1f}")
    for c, d in sorted(ages.items(), key=lambda kv: -kv[1])[:a.oldest]:
        print(f"  {d:5.1f} days  {owner(c):9} {c}")
    old = [c for c, d in ages.items() if a.max_days is not None and d > a.max_days]
    return 1 if old else 0


if __name__ == "__main__":
    sys.exit(main())
