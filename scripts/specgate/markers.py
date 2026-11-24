"""A diagnostic case marks the line its diagnostic points at (decision 473).

`// expect: E0601` plus a program line ending in `// <- here`; `--fix`
rewrites an old `// expect: E0601 at N` case, where N counted from the first
line after the header directives.
"""
from __future__ import annotations

import re

from specgate import Problem, case_id, cases, read, write

HELP = "every `// expect: CODE` case marks its line with `// <- here`, none counts lines"
HERE = "// <- here"
OLD = re.compile(r"^// expect: (\S+) at (\d+)((?: \(\w+\))?)\s*$")
# The directives the old `at N` count skipped.
SKIPPED = ("expect:", "rule:", "input:", "tool:", "file:", "has:", "lacks:", "rewritten:", "timeout:")


def args(p):
    p.add_argument("--fix", action="store_true", help="rewrite `at N` cases to the marker")


def migrate(text: str) -> str | None:
    lines = text.split("\n")
    m = OLD.match(lines[0].strip())
    if not m:
        return None
    offset = next(
        (i for i, l in enumerate(lines)
         if not (l.lstrip().startswith("//") and l.lstrip()[2:].lstrip().startswith(SKIPPED))),
        len(lines),
    )
    target = offset + int(m.group(2)) - 1
    if target >= len(lines):
        return None
    lines[0] = f"// expect: {m.group(1)}{m.group(3)}"
    line = lines[target]
    cr = "\r" if line.endswith("\r") else ""
    body = line.removesuffix(cr).rstrip()
    lines[target] = (f"{body} {HERE}" if body else HERE) + cr
    return "\n".join(lines)


def run(a) -> list[Problem]:
    problems = []
    for path in cases():
        text = read(path)
        if not OLD.match(text.split("\n", 1)[0].strip()):
            continue
        new = migrate(text)
        if a.fix and new is not None:
            write(path, new)
            continue
        problems.append(Problem(case_id(path), "counts lines with `at N`; run `scripts/spec-gate.py markers --fix`"))
    return problems
