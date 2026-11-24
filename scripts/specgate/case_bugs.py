"""Enforces conformance/R3 (handbook/guidelines/conformance-checklist.md#rules). Flag failing conformance cases whose failure looks like a bug in the case.

A known-gap case is expected to fail, so a wrong header or a stale API use
in it hides until the gap closes. This runs the suite (or reads a saved
`cpl-conform conformance --verbose` log) and flags failures that point at
the case rather than the compiler:

  same-code  the case expects code X and the run reports code X, so its
             `// <- here` line or `has:` is wrong
  optional   the run reports E0606 because a `?T` (now returned by min, max,
             pop and the searches) is printed without `!` or `??`

Enforces handbook/guidelines/conformance-checklist.md#rules (conformance/R3)
"""
from __future__ import annotations

import re
import subprocess
from pathlib import Path

import specgate
from specgate import Problem

HELP = "no failing conformance case fails because of the case (same code as wanted, a printed ?T)"
ANSI = re.compile(r"\x1b\[[0-9;]*m")
LINE = re.compile(r"^\s*(\S+\.cp): fail: wanted (.*?); got: (.*)$")
WANT_CODE = re.compile(r"^(?:trap )?([EWLR]\d{4})\b")
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


def args(p) -> None:
    p.add_argument("--log")
    p.add_argument("filter", nargs="?")


def run(a) -> list[Problem]:
    if a.log:
        log = Path(a.log).read_text()
    else:
        cmd = [str(specgate.ROOT / "target/debug/cpl-conform"), "conformance", "--verbose", *([a.filter] if a.filter else [])]
        log = subprocess.run(cmd, cwd=specgate.ROOT, capture_output=True, text=True).stdout
    return [Problem(path, f"{kind}: {got}") for path, kind, got in suspects(log)]
