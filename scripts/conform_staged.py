#!/usr/bin/env python3
"""Run the staged spec/conformance cases through cpl-conform (commit tier).

    scripts/conform_staged.py CPL_CONFORM CASE...

A staged case that fails blocks the commit unless HEAD's
research/scoreboard.json already records it as failing: no new failing case,
and no marker line in the case (`conformance_failing` counts the board's
failures, target 0). cpl-conform's message names the fix, e.g. the line to
mark `// <- here` (conformance/R1, conformance/R3). Each case runs twice, and one whose verdict differs
between the runs is a flake and blocks too.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PREFIX = "spec/conformance/"
BOARD = "research/scoreboard.json"
FAIL = re.compile(r"^  (\S+\.cp): fail")


def board_failing(board_text):
    """Case ids the scoreboard JSON `board_text` records as not passing."""
    if not board_text:
        return set()
    cases = json.loads(board_text).get("suites", {}).get("conformance", {}).get("cases", {})
    return {c for c, v in cases.items() if v != "pass"}


def head_board():
    """HEAD's scoreboard, so staging a board edit can't excuse a failure."""
    r = subprocess.run(["git", "show", f"HEAD:{BOARD}"], cwd=ROOT, capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else None


def new_failures(case, output, known):
    """The failure report for `case` in cpl-conform's --verbose output, unless
    the scoreboard already records it as failing."""
    if case in known:
        return []
    lines, out = output.splitlines(), []
    for i, line in enumerate(lines):
        m = FAIL.match(line)
        if m and m[1] == case:
            out.append(line)
            out += [l for l in lines[i + 1:i + 12] if l.startswith("  ") and not FAIL.match(l)]
    return out


def verdict(case, output):
    """`fail` when cpl-conform's --verbose output fails `case`, else `pass`."""
    return "fail" if any((m := FAIL.match(l)) and m[1] == case for l in output.splitlines()) else "pass"


def flake(case, first, second):
    """The report for a case whose two runs disagree, else nothing."""
    a, b = verdict(case, first), verdict(case, second)
    return [] if a == b else [f"  {case}: fail: flaky, {a} then {b} on the same build"]


def main(argv):
    conform, paths = argv[0], argv[1:]
    known = board_failing(head_board())
    bad = []
    for path in paths:
        if not (ROOT / path).is_file():
            continue
        case = path.removeprefix(PREFIX)
        runs = [subprocess.run([conform, "conformance", "--verbose", case], cwd=ROOT, capture_output=True, text=True)
                for _ in range(2)]
        first, second = (r.stdout + r.stderr for r in runs)
        bad += new_failures(case, first, known)
        bad += flake(case, first, second)
    if bad:
        print("\n".join(bad))
        print(f"conform_staged: {sum(bool(FAIL.match(l)) for l in bad)} staged case(s) fail that {BOARD} records as passing or new; fix them")
        return 1
    print(f"conform_staged: {len(paths)} staged case(s) pass or were already failing")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
