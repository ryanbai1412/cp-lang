#!/usr/bin/env python3
# Enforces handbook/guidelines/spec-rules-checklist.md#spec-rules7-a-removed-spelling-is-in-the-changelog
"""Fail if a spelling the spec removes (E0503) is missing from the changelog.

Reads the `old` is `new` pairs in spec/diagnostics.md's E0503 row and
checks that each old spelling appears in CHANGELOG.md's first (current)
version section, where the release notes come from.

    uv run scripts/spec-changelog.py
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    row = next(l for l in (ROOT / "spec" / "diagnostics.md").read_text().splitlines() if l.startswith("| E0503"))
    pairs = re.findall(r"`([^`]+)` is `([^`]+)`", row)
    section = (ROOT / "CHANGELOG.md").read_text().split("\n## ")[1]
    missing = [(o, n) for o, n in pairs if f"`{o}`" not in section]
    print(f"spec-changelog: {len(pairs)} removed spellings, {len(missing)} missing from the changelog's current section")
    for o, n in missing:
        print(f"  `{o}` (now `{n}`): add it under Breaking changes in CHANGELOG.md")
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
