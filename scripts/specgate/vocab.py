"""No spelling spec/GLOSSARY.md bans, across spec/core, spec/lib, std and the lint records."""
from __future__ import annotations

import re

import specgate
from specgate import Problem

HELP = "no spelling spec/GLOSSARY.md bans in spec/core, spec/lib, std or spec/lints"

BANNED = [
    (r"\bMint\b(?!`\))", "write `Mod<P>`", r"not `Mint`"),
    (r"->\s*Order\b(?!ing)", "`cmp` returns `Ordering`", None),
    (r"\)\s+lends\s", "write `-> lends T`", None),
    (r"#lint\b", "a library declares a lint with `@lint(name, from:, to:)` (R06.4.9)", None),
    (r"\bpub\s+@get\b", "decorators come before `pub`: `@get pub fn`", None),
    (r"\bsome\(", "the variant is `Some`", None),
    (r"(?<![\"'\w])none(?=\s*(\)|,|$|\?\?| else| =>|\bor\b))", "the variant is `None`",
     r'none:|"none"|none possible|none of|or none'),
    (r"^\s*(pub\s+)?enum\s+\w+\s*:\s*\w+\s*,", "one-line enums are `enum E = A | B`", None),
]
GLOBS = ["spec/core/*.md", "spec/lib/**/*.md", "std/*.cp", "spec/lints/*/*.cp", "spec/lints/*/*.md"]


def args(p) -> None:
    pass


def code_lines(path):
    """(line number, line, is code) for a std/lint `.cp` (code unless a `///` doc) or a doc (code in a fence)."""
    text = path.read_text().split("\n")
    if path.suffix == ".cp":
        for i, line in enumerate(text, 1):
            yield i, line, "///" not in line
        return
    fence = False
    for i, line in enumerate(text, 1):
        if line.lstrip().startswith("```"):
            fence = not fence
            continue
        yield i, line, fence


def run(a) -> list[Problem]:
    root = specgate.ROOT
    out = []
    for f in sorted({p for g in GLOBS for p in root.glob(g)}):
        for i, line, code in code_lines(f):
            for pat, why, allow in BANNED:
                if why.startswith("the variant") and not code:
                    continue
                if re.search(pat, line) and not (allow and re.search(allow, line)):
                    out.append(Problem(f"{f.relative_to(root)}:{i}", f"{why}: {line.strip()[:90]}"))
    return out
