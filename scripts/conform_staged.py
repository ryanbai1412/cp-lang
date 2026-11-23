#!/usr/bin/env python3
"""Run the staged spec/conformance cases through cpl-conform (commit tier).

    scripts/conform_staged.py CPL_CONFORM CASE...

A case that fails blocks the commit unless research/scoreboard.json already
records it as failing. cpl-conform's message names the fix, e.g. the right
`at N`.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PREFIX = "spec/conformance/"
FAIL = re.compile(r"^  (\S+\.cp): fail")


def new_failures(case, output, known):
    """The failure report for `case` in cpl-conform's --verbose output, unless
    the scoreboard already lists the case as failing."""
    lines, out = output.splitlines(), []
    for i, line in enumerate(lines):
        m = FAIL.match(line)
        if m and m[1] == case and known.get(case) != "fail":
            out.append(line)
            out += [l for l in lines[i + 1:i + 12] if l.startswith("  ") and not FAIL.match(l)]
    return out


def main(argv):
    conform, paths = argv[0], argv[1:]
    known = json.loads((ROOT / "research" / "scoreboard.json").read_text())["suites"]["conformance"]["cases"]
    bad = []
    for path in paths:
        if not (ROOT / path).is_file():
            continue
        case = path.removeprefix(PREFIX)
        r = subprocess.run([conform, "conformance", "--verbose", case], cwd=ROOT, capture_output=True, text=True)
        bad += new_failures(case, r.stdout + r.stderr, known)
    if bad:
        print("\n".join(bad))
        print(f"conform_staged: {sum(bool(FAIL.match(l)) for l in bad)} staged case(s) fail; fix them, or record a known gap as failing in research/scoreboard.json")
        return 1
    print(f"conform_staged: {len(paths)} staged case(s) pass")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
