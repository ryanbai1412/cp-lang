"""R11.3.2: every free pub fn in a spec/lib chapter's tables is a prelude name.

A table entry is a top-level `fn name` line in a fenced block of spec/lib/*.md,
or a Markdown table row starting with a `name(...)` cell, whose name std
declares as a top-level `pub fn` taking no `self` (a `self` fn is a method and
needs no import, R11.3.4). Each must be imported by std/prelude.cp. A chapter
that means a qualified name spells it `fn module.name` (R11.3.3); std/<module>.cp
must declare it as a free `pub fn`, and the prelude must not import it bare.
(`scripts/spec-gate.py std-docs` checks the other way: prelude names against
the chapters.) There is no known-gap list: every miss fails.
Rule: handbook/process/quality.md#which-test-to-write.
Enforces handbook/guidelines/conformance-checklist.md#conformancer5-where-a-rule-covers-a-space-generate-the-cases
"""
from __future__ import annotations

import re

import specgate
from specgate import Problem

HELP = "every free spec/lib table fn std declares is a prelude name; a qualified one is never imported bare"


def args(p) -> None:
    pass


def prelude_imports(root) -> set[tuple[str, str]]:
    return {(module, name)
            for module, group in re.findall(r"use std\.(\w+)\.\{([^}]*)\}", (root / "std/prelude.cp").read_text())
            for name in re.findall(r"\b\w+\b", group)}


def std_free_fns(root) -> dict[str, set[str]]:
    return {p.stem: {name for name, rest in re.findall(r"^pub fn (\w+)(.*)$", p.read_text(), re.M)
                     if not re.search(r"\((?:mut\s+)?self\b", rest)}
            for p in root.glob("std/*.cp")}


def table_fns(text: str):
    for body in re.findall(r"^```(?:cp|python)\s*\n(.*?)^```", text, re.S | re.M):
        yield from re.findall(r"^fn (?:(\w+)\.)?(\w+)\b(?!\.)", body, re.M)
    yield from re.findall(r"^\| `(?:(\w+)\.)?(\w+)[(<]", text, re.M)


def misses(root) -> list[Problem]:
    imports, mods = prelude_imports(root), std_free_fns(root)
    have = {name for _, name in imports}
    pub = set().union(*mods.values())
    found: set[tuple[str, str]] = set()
    for p in sorted(root.glob("spec/lib/*.md")):
        where = str(p.relative_to(root))
        for module, name in table_fns(p.read_text()):
            if not module:
                if name in pub and name not in have:
                    found.add((where, f"`{name}` is a table fn std/prelude.cp does not import"))
            elif module in mods:
                if name not in mods[module]:
                    found.add((where, f"`{module}.{name}`: std/{module}.cp has no free `pub fn {name}`"))
                elif (module, name) in imports:
                    found.add((where, f"`{module}.{name}` is written qualified, but std/prelude.cp imports it bare"))
    return [Problem(w, what) for w, what in sorted(found)]


def run(a) -> list[Problem]:
    return misses(specgate.ROOT)
