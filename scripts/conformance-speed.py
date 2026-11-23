#!/usr/bin/env python3
# Enforces handbook/guidelines/conformance-checklist.md#conformancer8-the-suite-stays-fast-enough-to-run-on-every-land
"""Fail if the conformance and spec-block suites cost more CPU than the land budget.

Runs `cpl-conform conformance spec-blocks` and measures the CPU seconds
(user + sys) it and its children spend. CPU time, unlike wall time, barely
moves with machine load, so the budget is a gate, not a metric. The budget
is about twice today's cost: a doubling means a slow case or a harness
regression, and the run prints where the time went so it can be found.

    uv run scripts/conformance-speed.py [--budget SECS] [--cpl-conform PATH]
"""
import argparse
import resource
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BUDGET = 260.0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--budget", type=float, default=BUDGET)
    ap.add_argument("--cpl-conform", default=str(ROOT / "target" / "debug" / "cpl-conform"))
    a = ap.parse_args()
    start = time.monotonic()
    r = subprocess.run([a.cpl_conform, "conformance", "spec-blocks"], cwd=ROOT, capture_output=True, text=True)
    wall = time.monotonic() - start
    ru = resource.getrusage(resource.RUSAGE_CHILDREN)
    cpu = ru.ru_utime + ru.ru_stime
    print(f"conformance-speed: {cpu:.0f} CPU s ({wall:.0f} s wall) for conformance + spec-blocks; budget {a.budget:.0f} CPU s")
    if r.returncode not in (0, 1):
        print(r.stderr[-2000:])
        return 1
    if cpu > a.budget:
        print("conformance-speed: over budget; find the slow case with `cpl-conform conformance --verbose` and a per-dir timing")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
