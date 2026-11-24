"""Every tool feature in spec/tools/ carries a rule id.

A feature is a top-level bullet with its continuation lines. It has a rule
id when one ends a sentence: `(Rnn.n.n)` as in spec/core or a tool id such
as `(Rtrace.4)`, which cases cite as `trace.4`. Without one, no case can cite
it and the coverage gate can't see it.
Enforces handbook/guidelines/spec-rules-checklist.md#spec-rulesr8-every-tool-feature-has-a-rule-id
"""
from __future__ import annotations

import re

import specgate
from specgate import Problem

HELP = "every top-level bullet in spec/tools/ ends a sentence with a rule id"
RULE = re.compile(r"\(R(?:\d\d[a-z]?|[a-z]+)(?:\.\d+)+(?:, permits)?\)")


def args(p) -> None:
    pass


def features(path):
    item, out, fence = None, [], False
    for i, line in enumerate(path.read_text().splitlines(), 1):
        if line.startswith("```"):
            fence = not fence
        if fence:
            continue
        if line.startswith("- "):
            if item:
                out.append(item)
            item = [i, line]
        elif item and line.startswith("  ") and line.strip():
            item[1] += " " + line.strip()
        elif item:
            out.append(item)
            item = None
    if item:
        out.append(item)
    return out


def run(a) -> list[Problem]:
    root = specgate.ROOT
    return [Problem(f"{f.relative_to(root)}:{line}", f"no rule id: {text[:90]}")
            for f in sorted((root / "spec" / "tools").glob("*.md"))
            for line, text in features(f) if not RULE.search(text)]
