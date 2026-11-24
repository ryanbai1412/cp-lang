"""Every decision cited in spec/ is explained by a handbook doc that names it.

A rule citing `decision N` rests on a design call whose reason and losing
alternatives a reader should find in the handbook.
Enforces handbook/guidelines/spec-rules-checklist.md#spec-rulesr6-the-handbook-and-the-spec-change-together
"""
from __future__ import annotations

import re

import specgate
from specgate import Problem

HELP = "every decision cited in spec/ is named by a handbook doc"
CITE = re.compile(r"[Dd]ecisions? (\d{2,4})")


def args(p) -> None:
    pass


def cited(root, paths) -> dict[str, set[str]]:
    out: dict[str, set[str]] = {}
    for p in paths:
        for n in CITE.findall(p.read_text()):
            out.setdefault(n, set()).add(str(p.relative_to(root)))
    return out


def run(a) -> list[Problem]:
    root = specgate.ROOT
    spec = cited(root, (root / "spec").rglob("*.md"))
    hb = cited(root, (root / "handbook").rglob("*.md"))
    return [Problem(f"decision {n}", f"cited in {', '.join(sorted(spec[n]))}; explain it in handbook/language/")
            for n in sorted(set(spec) - set(hb), key=int)]
