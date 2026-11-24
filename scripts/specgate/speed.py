"""The conformance and spec-block suites cost at most the land budget in CPU.

CPU time (user + sys of the run and its children), unlike wall time, barely
moves with machine load, so the budget is a gate. It is about twice today's
cost: a doubling means a slow case or a harness regression.
Enforces handbook/guidelines/conformance-checklist.md#conformancer8-the-suite-stays-fast-enough-to-run-on-every-land
"""
from __future__ import annotations

import resource
import subprocess
import time

import specgate
from specgate import Problem

HELP = "cpl-conform conformance spec-blocks stays within its CPU budget"
BUDGET = 260.0


def args(p) -> None:
    p.add_argument("--budget", type=float, default=BUDGET)
    p.add_argument("--cpl-conform", default=None)


def run(a) -> list[Problem]:
    conform = a.cpl_conform or str(specgate.ROOT / "target" / "debug" / "cpl-conform")
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    start = time.monotonic()
    r = subprocess.run([conform, "conformance", "spec-blocks"], cwd=specgate.ROOT, capture_output=True, text=True)
    wall = time.monotonic() - start
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    cpu = after.ru_utime + after.ru_stime - before.ru_utime - before.ru_stime
    if r.returncode not in (0, 1):
        return [Problem(conform, f"exited {r.returncode}: {r.stderr[-2000:]}")]
    if cpu > a.budget:
        return [Problem("conformance + spec-blocks", f"{cpu:.0f} CPU s ({wall:.0f} s wall), over the {a.budget:.0f} CPU s budget; "
                        "find the slow case with `cpl-conform conformance --verbose` and a per-dir timing")]
    return []
