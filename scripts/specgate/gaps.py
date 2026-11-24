"""`// gap:` lines are going (the debt ratchet counts them to 0): research/scoreboard.json
is the one record of a known failure, and `conformance_failing` counts it.

A failing case needs no line. A line left is well formed and its case still
fails; no reason text. The time is when the gap was opened (UTC, to the hour); the
`gap-age` gate fails a gap past 48 h. Results come from `--board` (a
scoreboard JSON) or from a fresh run of `--conform <cpl-conform>`.
"""
from __future__ import annotations

import datetime
import json
import pathlib
import subprocess
import tempfile
import re

import specgate
from specgate import Problem, case_id, cases, header, read

HELP = "every `// gap:` line left is well formed and its case still fails"
OWNERS = ("compiler", "stdlib", "diagnostics", "tools", "lint-fmt", "perf", "debugger", "site", "infra", "idioms")
GAP = re.compile(r"^// gap: (\S+) (\d{4}-\d\d-\d\dT\d\dZ)$")


def gap_line(text: str) -> str | None:
    lines = [l for l in header(text) if l.startswith("// gap:")]
    return lines[0] if lines else None


def parse(line: str):
    """`(owner, opened)` of a well-formed gap line, else a problem string."""
    m = GAP.match(line)
    if not m:
        return "is not `// gap: <owner> <YYYY-MM-DDTHHZ>` (no reason text: addendum 3)"
    if m.group(1) not in OWNERS:
        return f"owner `{m.group(1)}` is not one of {', '.join(OWNERS)}"
    try:
        at = datetime.datetime.strptime(m.group(2), "%Y-%m-%dT%HZ").replace(tzinfo=datetime.timezone.utc)
    except ValueError:
        return f"`{m.group(2)}` is not a time"
    return m.group(1), at


def all_gaps() -> dict[str, tuple[str, object]]:
    """case id -> (gap line, parse result) for every case with a gap line."""
    out = {}
    for path in cases():
        text = read(path)
        line = gap_line(text)
        if line is not None:
            out[case_id(path)] = (line, parse(line))
        if sum(l.startswith("// gap:") for l in header(text)) > 1:
            out[case_id(path)] = (line, "has more than one `// gap:` line")
    return out


def args(p):
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument("--board", help="scoreboard JSON with today's verdicts")
    g.add_argument("--conform", help="cpl-conform binary to run the suite with")


def board(a) -> dict[str, str]:
    if a.board:
        return json.loads(pathlib.Path(a.board).read_text())["suites"]["conformance"]["cases"]
    with tempfile.TemporaryDirectory() as tmp:
        out = pathlib.Path(tmp) / "board.json"
        # The suite exits non-zero when a case fails; the verdicts are what this gate judges.
        subprocess.run([a.conform, "conformance", "--write", "--scoreboard", str(out)],
                       cwd=specgate.ROOT, capture_output=True)
        return json.loads(out.read_text())["suites"]["conformance"]["cases"]


def run(a) -> list[Problem]:
    status = board(a)
    gaps = all_gaps()
    problems = [Problem(c, f"`{line}` {p}") for c, (line, p) in gaps.items() if isinstance(p, str)]
    for c in sorted(gaps):
        if status.get(c) == "pass":
            problems.append(Problem(c, "passes; delete its `// gap:` line"))
    return problems
