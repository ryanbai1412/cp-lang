"""No conformance gap is older than 48 hours, and none is re-dated.

A gap's time (`// gap: <owner> <YYYY-MM-DDTHHZ>`) is when it opened; past
48 h this gate fails until the case passes and the line goes. `--staged`
also rejects a staged edit that moves an existing gap's time later.
"""
from __future__ import annotations

import datetime
import subprocess

import specgate
from specgate import Problem
from specgate.gaps import all_gaps, gap_line, parse

HELP = "every `// gap:` is under 48 h old and never moved later"
MAX = datetime.timedelta(hours=48)


def args(p):
    p.add_argument("--now", help="the time to judge against (YYYY-MM-DDTHHZ); default now")
    p.add_argument("--staged", action="store_true", help="also compare staged gap times with HEAD's")


def head_text(rel: str) -> str | None:
    r = subprocess.run(["git", "show", f"HEAD:{rel}"], cwd=specgate.ROOT, capture_output=True)
    return r.stdout.decode() if r.returncode == 0 else None


def run(a) -> list[Problem]:
    now = (datetime.datetime.strptime(a.now, "%Y-%m-%dT%HZ").replace(tzinfo=datetime.timezone.utc)
           if a.now else datetime.datetime.now(datetime.timezone.utc))
    problems = []
    for c, (line, p) in sorted(all_gaps().items()):
        if isinstance(p, str):
            problems.append(Problem(c, f"`{line}` {p}"))
            continue
        owner, opened = p
        if opened > now + datetime.timedelta(hours=1):
            problems.append(Problem(c, f"gap opened in the future ({line})"))
        elif now - opened > MAX:
            hours = int((now - opened).total_seconds() // 3600)
            problems.append(Problem(c, f"{owner} gap is {hours} h old (max 48 h): make the case pass"))
        if a.staged:
            old = head_text(f"{specgate.CONFORMANCE.relative_to(specgate.ROOT).as_posix()}/{c}")
            was = parse(gap_line(old)) if old and gap_line(old) else None
            if isinstance(was, tuple) and was[1] < opened:
                problems.append(Problem(c, f"moves the gap's time later than {was[1]:%Y-%m-%dT%HZ}; keep the original"))
    return problems
