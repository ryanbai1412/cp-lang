"""No case carries a `// gap:` line (decision 527).

research/scoreboard.json is the one record of a known failure: a failing case
is a gap because the board says it fails, and `conformance_failing` counts
them. A line in the case would be a second record that drifts from the first.
"""
from __future__ import annotations

from specgate import Problem, case_id, cases, header, read

HELP = "no case carries a `// gap:` line; the scoreboard is the one record of a known failure"


def args(p):
    pass


def run(a) -> list[Problem]:
    return [Problem(case_id(path), "has a `// gap:` line; delete it (the scoreboard records the failure)")
            for path in cases() if any(l.startswith("// gap:") for l in header(read(path)))]
