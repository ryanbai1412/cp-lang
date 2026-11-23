#!/usr/bin/env python3
"""Enforces handbook/guidelines/conformance.md#c3. Flag failing conformance cases whose failure looks like a bug in the case.

A known-gap case is expected to fail, so a wrong header or a stale API use
in it hides until the gap closes. This runs the suite (or reads a saved
`cpl-conform conformance --verbose` log) and flags failures that point at
the case rather than the compiler:

  same-code  the case expects code X and the run reports code X, so its
             `at N` or `has:` is wrong
  optional   the run reports E0606 because a `?T` (now returned by min, max,
             pop and the searches) is printed without `!` or `??`

Usage:
    python3 scripts/conformance-gaps.py [--log FILE] [--check] [filter]
"""
import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ANSI = re.compile(r"\x1b\[[0-9;]*m")
LINE = re.compile(r"^\s*(\S+\.cp): fail: wanted (.*?); got: (.*)$")
WANT_CODE = re.compile(r"^(?:trap )?([EWLR]\d{4}) at \d+")
GOT_CODE = re.compile(r"^error\[([EWLR]\d{4})\]")
OPTIONAL = re.compile(r"E0606\]: `[^`]*` has type `\?")


def classify(want: str, got: str) -> str | None:
    w, g = WANT_CODE.match(want), GOT_CODE.match(got)
    if w and g and w.group(1) == g.group(1):
        return "same-code"
    if OPTIONAL.search(got):
        return "optional"
    return None


def suspects(log: str) -> list[tuple[str, str, str]]:
    out = []
    for raw in log.splitlines():
        m = LINE.match(ANSI.sub("", raw))
        if m and (kind := classify(m.group(2), m.group(3))):
            out.append((m.group(1), kind, m.group(3)[:160]))
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--log")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("filter", nargs="?")
    a = ap.parse_args()
    if a.log:
        log = Path(a.log).read_text()
    else:
        cmd = [str(ROOT / "target/debug/cpl-conform"), "conformance", "--verbose"]
        if a.filter:
            cmd.append(a.filter)
        log = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True).stdout
    found = suspects(log)
    for path, kind, got in found:
        print(f"{path}: {kind}: {got}")
    print(f"{len(found)} failing cases look like case bugs")
    return 1 if a.check and found else 0


if __name__ == "__main__":
    sys.exit(main())
