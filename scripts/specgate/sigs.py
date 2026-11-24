"""Check that named arguments in spec/core and spec/lib code blocks exist in std.

For every call `f(..., k: v)` in a fenced block where some `fn f` is declared in
std, `k` must be a parameter of at least one std `fn f`. Calls to names that
std does not declare (user fns, records, constructors) are not checked.
Any mismatch fails: fix std or the chapter.

Enforces handbook/guidelines/conformance-checklist.md#conformancer5-where-a-rule-covers-a-space-generate-the-cases
"""
from __future__ import annotations

import glob
import os
import re
from collections import defaultdict

import specgate
from specgate import Problem

HELP = "every named argument in a spec/core or spec/lib call to a std fn is a parameter of it"


def balanced(text, i):
    """Return the text inside the parens opening at text[i] == '('."""
    depth = 0
    for j in range(i, len(text)):
        c = text[j]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                return text[i + 1 : j]
    return None


def top_level_parts(s, angle=False):
    opens, closes = ("([{<", ")]}>") if angle else ("([{", ")]}")
    parts, depth, cur, prev = [], 0, "", ""
    for c in s:
        arrow = c == ">" and prev in "=-"
        prev = c
        if c in opens:
            depth += 1
        elif c in closes and not arrow:
            depth -= 1
        if c == "," and depth == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += c
    parts.append(cur)
    return [p.strip() for p in parts if p.strip()]


def std_params():
    """Parameter names per std fn name: (any fn, top-level fns only). A bare call `f(...)` can
    only reach a top-level fn; a method of the same name (`Mod.read`) is not what it calls."""
    params, top = defaultdict(set), defaultdict(set)
    for path in glob.glob(os.path.join(str(specgate.ROOT), "std/*.cp")):
        text = open(path).read()
        for m in re.finditer(r"^([ \t]*)[^\n]*?\bfn\s+(\w+)\s*(<[^(\n]*>)?\s*\(", text, re.M):
            inner = balanced(text, m.end() - 1)
            if inner is None:
                continue
            names = {""}
            for p in top_level_parts(inner, angle=True):
                if p.strip().startswith(".."):
                    names.add("*")  # a variadic `..xs` takes any name (R06.1.5, decision 324)
                n = re.match(r"(?:mut\s+)?(\w+)\s*[:=]", p)
                if n:
                    names.add(n.group(1))
            params[m.group(2)] |= names
            if not m.group(1):
                top[m.group(2)] |= names
    return params, top


def blocks(path):
    lines = open(path).read().split("\n")
    i = 0
    while i < len(lines):
        if re.match(r"^\s*```(cp|python)\s*$", lines[i]):
            start, body = i + 1, []
            i += 1
            while i < len(lines) and not re.match(r"^\s*```\s*$", lines[i]):
                body.append(lines[i])
                i += 1
            yield start, "\n".join(body)
        i += 1


def mismatches():
    any_fn, top = std_params()
    out = set()
    for path in sorted(glob.glob(os.path.join(str(specgate.ROOT), "spec/core/*.md")) + glob.glob(os.path.join(str(specgate.ROOT), "spec/lib/*.md"))):
        rel = os.path.relpath(path, os.path.join(str(specgate.ROOT), "spec"))
        for _, body in blocks(path):
            code = re.sub(r"//[^\n]*", "", body)
            for m in re.finditer(r"(\.?)\b(\w+)\s*\(", code):
                name = m.group(2)
                params = any_fn if m.group(1) else top
                if name not in params or re.search(r"\b(fn|law)\s+$", code[: m.start()][-8:]):
                    continue
                inner = balanced(code, m.end() - 1)
                if inner is None:
                    continue
                for p in top_level_parts(inner):
                    k = re.match(r"(\w+):(?!:)", p)
                    if k and k.group(1) not in params[name] and "*" not in params[name]:
                        out.add(f"{rel}: {m.group(1)}{name}({k.group(1)}:)")
    return out


def args(p) -> None:
    pass


def run(a) -> list[Problem]:
    return [Problem(*line.split(": ", 1)) if ": " in line else Problem("spec", line) for line in sorted(mismatches())]
