#!/usr/bin/env python3
"""Run the staged spec/conformance cases through cpl-conform (commit tier).

    scripts/conform_staged.py CPL_CONFORM CASE...

A case that fails blocks the commit unless it carries a
`// gap: <owner> <time>` line (decision 515), and a gap case that passes blocks
it until the line goes (conformance/R1, conformance/R3). cpl-conform's message names the fix, e.g. the line
to mark `// <- here`.
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PREFIX = "spec/conformance/"
FAIL = re.compile(r"^  (\S+\.cp): fail")
PASSED = re.compile(r"^conformance\s+(\d+)/\1\s", re.M)


def is_gap(text):
    """Whether the case's header has a `// gap:` line."""
    for line in text.split("\n"):
        if not line.lstrip().startswith("//"):
            return False
        if line.lstrip()[2:].lstrip().startswith("gap:"):
            return True
    return False


def new_failures(case, output, gap):
    """The failure report for `case` in cpl-conform's --verbose output, unless
    the case is a known gap; a gap case that passes is reported too."""
    lines, out = output.splitlines(), []
    failed = False
    for i, line in enumerate(lines):
        m = FAIL.match(line)
        if m and m[1] == case:
            failed = True
        if m and m[1] == case and not gap:
            out.append(line)
            out += [l for l in lines[i + 1:i + 12] if l.startswith("  ") and not FAIL.match(l)]
    if gap and not failed and PASSED.search(output):
        out.append(f"  {case}: fail: passes now; delete its `// gap:` line")
    return out


def main(argv):
    conform, paths = argv[0], argv[1:]
    bad = []
    for path in paths:
        if not (ROOT / path).is_file():
            continue
        case = path.removeprefix(PREFIX)
        r = subprocess.run([conform, "conformance", "--verbose", case], cwd=ROOT, capture_output=True, text=True)
        bad += new_failures(case, r.stdout + r.stderr, is_gap((ROOT / path).read_bytes().decode()))
    if bad:
        print("\n".join(bad))
        print(f"conform_staged: {sum(bool(FAIL.match(l)) for l in bad)} staged case(s) fail; fix them, or open a gap: `// gap: <owner> <YYYY-MM-DDTHHZ>` under `// expect:`")
        return 1
    print(f"conform_staged: {len(paths)} staged case(s) pass")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
