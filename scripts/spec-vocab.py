#!/usr/bin/env python3
"""Fail on spellings that spec/GLOSSARY.md bans, across spec/core, spec/lib, std and the lint records."""
import glob, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BANNED = [
    (r"\bMint\b(?!`\))", "write `Mod<P>`", r"not `Mint`"),
    (r"->\s*Order\b(?!ing)", "`cmp` returns `Ordering`", None),
    (r"\)\s+lends\s", "write `-> lends T`", None),
    (r"#lint\b", "a library declares a lint with `@lint(name, from:, to:)` (R06.4.9)", None),
    (r"\bpub\s+@get\b", "decorators come before `pub`: `@get pub fn`", None),
    (r"\bsome\(", "the variant is `Some`", None),
    (r"(?<![\"'\w])none(?=\s*(\)|,|$|\?\?| else| =>|\bor\b))", "the variant is `None`", r'none:|"none"|none possible|none of|or none'),
    (r"^\s*(pub\s+)?enum\s+\w+\s*:\s*\w+\s*,", "one-line enums are `enum E = A | B`", None),
]

def code_lines(path):
    text = open(path).read().split("\n")
    if path.endswith(".cp"):
        for i, line in enumerate(text, 1):
            yield i, line, "///" not in line
        return
    fence = False
    for i, line in enumerate(text, 1):
        if line.lstrip().startswith("```"):
            fence = not fence
            continue
        yield i, line, fence

bad = 0
files = sorted(glob.glob(f"{ROOT}/spec/core/*.md") + glob.glob(f"{ROOT}/spec/lib/**/*.md", recursive=True)
               + glob.glob(f"{ROOT}/std/*.cp") + glob.glob(f"{ROOT}/spec/lints/*/*.cp")
               + glob.glob(f"{ROOT}/spec/lints/*/*.md"))
for f in files:
    for i, line, code in code_lines(f):
        for pat, why, allow in BANNED:
            if why.startswith("the variant") and not code:
                continue
            if re.search(pat, line) and not (allow and re.search(allow, line)):
                print(f"{os.path.relpath(f, ROOT)}:{i}: {why}: {line.strip()[:90]}")
                bad += 1
print(f"spec-vocab: {bad} banned spelling(s)")
sys.exit(1 if bad else 0)
