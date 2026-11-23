#!/usr/bin/env python3
"""Report which corpus ports open with the standard header (plan/PORTING.md, "Header").

    uv run python corpus/header.py          count, then the 20 longest ports missing it
    uv run python corpus/header.py --all    list every port missing it
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
FIELDS = ["Source", "Problem", "Solution"]


def has_header(text):
    head = []
    for line in text.split("\n"):
        if not line.startswith("//"):
            break
        head.append(line)
    found = {m.group(1) for l in head for m in [re.match(r"//\s*(Source|Problem|Solution):", l)] if m}
    return all(f in found for f in FIELDS)


def main(argv):
    ports = sorted(ROOT.glob("problems/*/port.cp"))
    missing = [(len(p.read_text().split("\n")), p.parent.name) for p in ports if not has_header(p.read_text())]
    missing.sort(reverse=True)
    print(f"header: {len(ports) - len(missing)}/{len(ports)} ports have Source, Problem and Solution")
    for n, name in missing if "--all" in argv else missing[:20]:
        print(f"  {name} ({n} lines)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
