"""Every rule id written anywhere in the repo names a rule spec/core defines.

A rule is defined where `(Rnn.n.n)` ends a sentence in `spec/core/`; a
mention naming no defined rule is a dangling reference.
Enforces handbook/guidelines/spec-rules-checklist.md#spec-rulesr3-every-rule-has-an-id-every-reference-says-see
"""
from __future__ import annotations

import re
import subprocess

import specgate
from specgate import Problem

HELP = "every Rnn.n.n mention names a rule spec/core defines"
ID = r"R\d\d\.\d+\.\d+"
MENTION = re.compile(r"(?<![\w.])" + ID + r"(?![\w.]*\d)")
DEF = re.compile(r"\(((?:" + ID + r")(?:,\s*" + ID + r")*)\)")
SEARCHED = ("spec", "handbook", "std", "crates", "site", "editors", "plan", "decisions.md")
SUFFIXES = (".md", ".cp", ".rs", ".toml", ".txt", ".json", ".ts", ".js")


def args(p) -> None:
    pass


def defined(root) -> set[str]:
    ids: set[str] = set()
    for f in sorted((root / "spec/core").glob("*.md")):
        for m in DEF.finditer(f.read_text(encoding="utf-8")):
            ids.update(re.findall(ID, m.group(1)))
    return ids


def run(a) -> list[Problem]:
    root = specgate.ROOT
    ids = defined(root)
    files = subprocess.run(["git", "ls-files", *SEARCHED], cwd=root,
                           capture_output=True, text=True, check=True).stdout.split()
    out = []
    for name in files:
        if not name.endswith(SUFFIXES):
            continue
        try:
            text = (root / name).read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        for m in MENTION.finditer(text):
            if m.group(0) not in ids:
                out.append(Problem(f"{name}:{text.count(chr(10), 0, m.start()) + 1}", f"{m.group(0)} names no rule in spec/core"))
    return out
