"""Every spelling the spec removes (E0503) is in the changelog's current section.

Enforces handbook/guidelines/spec-rules-checklist.md#spec-rulesr7-a-removed-spelling-is-in-the-changelog
"""
from __future__ import annotations

import re

import specgate
from specgate import Problem

HELP = "every spelling spec/diagnostics.md's E0503 row removes is in CHANGELOG.md's current section"


def args(p) -> None:
    pass


def run(a) -> list[Problem]:
    root = specgate.ROOT
    row = next((l for l in (root / "spec" / "diagnostics.md").read_text().splitlines() if l.startswith("| E0503")), "")
    section = (root / "CHANGELOG.md").read_text().split("\n## ")[1]
    return [Problem("CHANGELOG.md", f"`{old}` (now `{new}`) is removed by E0503: add it under Breaking changes")
            for old, new in re.findall(r"`([^`]+)` is `([^`]+)`", row) if f"`{old}`" not in section]
