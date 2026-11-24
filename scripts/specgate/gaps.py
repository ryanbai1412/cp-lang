"""A case that fails today carries `// gap: <owner> <YYYY-MM-DDTHHZ>`, and only such a case.

The gap line is the one record of a known failure: no shared list, no
reason text. The time is when the gap was opened (UTC, to the hour); the
`gap-age` gate fails a gap past 48 h. Results come from `--board` (a
scoreboard JSON, default the committed `research/scoreboard.json`).
"""
from __future__ import annotations

import datetime
import json
import pathlib
import re

import specgate
from specgate import Problem, case_id, cases, header, read

HELP = "every failing case has a `// gap:` line and every gap line's case fails"
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
    p.add_argument("--board", default=str(specgate.ROOT / "research" / "scoreboard.json"),
                   help="scoreboard JSON with today's verdicts")


def run(a) -> list[Problem]:
    status = json.loads(pathlib.Path(a.board).read_text())["suites"]["conformance"]["cases"]
    gaps = all_gaps()
    problems = [Problem(c, f"`{line}` {p}") for c, (line, p) in gaps.items() if isinstance(p, str)]
    for c, verdict in sorted(status.items()):
        if verdict in ("fail", "bad", "timeout", "panic") and c not in gaps:
            problems.append(Problem(c, f"fails ({verdict}) with no `// gap: <owner> <time>` line; fix it or add one"))
    for c in sorted(gaps):
        if status.get(c) == "pass":
            problems.append(Problem(c, "passes; delete its `// gap:` line"))
    return problems
