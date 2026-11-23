#!/usr/bin/env python3
# Enforces handbook/guidelines/spec-rules-checklist.md#spec-rulesr3-every-rule-has-an-id-every-reference-says-see
"""Every rule id written anywhere in the repo names a rule the spec defines.

A rule is defined where `(Rnn.n.n)` ends a sentence in `spec/core/`. A
mention of `Rnn.n.n` in the spec, the handbook, std, the compiler, the tests
or the site that names no defined rule is a dangling reference: the reader
following it finds nothing.

    uv run scripts/spec-xref.py           # list dangling ids
    uv run scripts/spec-xref.py --check   # exit 1 if any
"""
import re
import subprocess
import sys
from pathlib import Path

ID = r"R\d\d\.\d+\.\d+"
MENTION = re.compile(r"(?<![\w.])" + ID + r"(?![\w.]*\d)")
DEF = re.compile(r"\(((?:" + ID + r")(?:,\s*" + ID + r")*)\)")
SEARCHED = ("spec", "handbook", "std", "crates", "site", "editors", "plan", "decisions.md")
SUFFIXES = (".md", ".cp", ".rs", ".toml", ".txt", ".json", ".ts", ".js")


def defined(root: Path) -> set[str]:
    ids = set()
    for f in sorted((root / "spec/core").glob("*.md")):
        for m in DEF.finditer(f.read_text(encoding="utf-8")):
            ids.update(re.findall(ID, m.group(1)))
    return ids


def dangling(root: Path) -> list[tuple[str, int, str]]:
    ids = defined(root)
    files = subprocess.run(["git", "ls-files", *SEARCHED], cwd=root,
                           capture_output=True, text=True, check=True).stdout.split()
    found = []
    for name in files:
        if not name.endswith(SUFFIXES):
            continue
        try:
            text = (root / name).read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        for m in MENTION.finditer(text):
            if m.group(0) not in ids:
                found.append((name, text.count("\n", 0, m.start()) + 1, m.group(0)))
    return found


def main(argv: list[str]) -> int:
    root = Path(__file__).resolve().parent.parent
    found = dangling(root)
    for name, line, rule in found:
        print(f"{name}:{line}: {rule} names no rule in spec/core")
    print(f"{len(found)} dangling rule references ({len(defined(root))} rules defined)")
    return 1 if found and "--check" in argv else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
