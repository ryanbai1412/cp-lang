#!/usr/bin/env python3
"""Allocate the next free lint code and scaffold its record in spec/lints/<name>/.

Usage:
    python3 scripts/new-lint.py NAME [--series 05] [--group idiom] [--level warn]
    python3 scripts/new-lint.py --next [--series 05]   # print the code only

A code counts as taken when any lint.toml under spec/lints/ or
spec/checker-lints/, or a row of spec/diagnostics.md, uses it, in the working
tree or on origin/main (fetched first, best effort), so a stale checkout does
not hand out a code another lane already landed. Two allocations that both
happen before either lands can still collide; `record_codes_are_unique` in
cpl-lint catches that, so land the record soon after allocating it.
"""
import argparse
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CODE_RE = re.compile(r'^code = "L(\d{4})"', re.MULTILINE)
ROW_RE = re.compile(r"^\| L(\d{4}) \|", re.MULTILINE)
NAME_RE = re.compile(r"^[a-z][a-z0-9_]*$")
FILES = ["before.cp", "after.cp", "near_miss.cp", "explain.md"]


def codes_in(text: str) -> set[int]:
    return {int(c) for c in CODE_RE.findall(text) + ROW_RE.findall(text)}


def next_code(used: set[int], series: int) -> int:
    lo = series * 100
    taken = [c for c in used if lo <= c < lo + 100]
    code = max(taken) + 1 if taken else lo + 1
    if code >= lo + 100:
        raise SystemExit(f"series L{series:02d}xx is full")
    return code


def git(*args: str) -> str:
    r = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else ""


def used_codes(fetch: bool = True) -> set[int]:
    used: set[int] = set()
    for sub in ("spec/lints", "spec/checker-lints"):
        base = os.path.join(ROOT, sub)
        for name in os.listdir(base) if os.path.isdir(base) else []:
            path = os.path.join(base, name, "lint.toml")
            if os.path.exists(path):
                with open(path, encoding="utf-8") as fh:
                    used |= codes_in(fh.read())
    with open(os.path.join(ROOT, "spec/diagnostics.md"), encoding="utf-8") as fh:
        used |= codes_in(fh.read())
    if fetch:
        git("fetch", "-q", "origin", "main")
    used |= codes_in(
        git("grep", "-h", "^code = ", "origin/main", "--", "spec/lints", "spec/checker-lints")
        .replace("origin/main:", "")
    )
    used |= codes_in(git("show", "origin/main:spec/diagnostics.md"))
    return used


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("name", nargs="?", help="snake_case lint name, also the record directory")
    p.add_argument("--series", type=int, default=5, help="hundreds of the code: 2, 4, 5, 6 (default 5)")
    p.add_argument("--group", default="idiom", help="correctness, perf, idiom, style or pedantic")
    p.add_argument("--level", default="warn", help="allow, warn or deny")
    p.add_argument("--next", action="store_true", help="print the next free code and stop")
    p.add_argument("--no-fetch", action="store_true", help="don't fetch origin/main first")
    a = p.parse_args(argv)
    code = f"L{next_code(used_codes(not a.no_fetch), a.series):04d}"
    if a.next:
        print(code)
        return 0
    if not a.name or not NAME_RE.match(a.name):
        print("error: give a snake_case NAME", file=sys.stderr)
        return 2
    rec = os.path.join(ROOT, "spec/lints", a.name)
    if os.path.exists(rec):
        print(f"error: {os.path.relpath(rec, ROOT)} already exists", file=sys.stderr)
        return 1
    os.makedirs(rec)
    with open(os.path.join(rec, "lint.toml"), "w", encoding="utf-8") as fh:
        fh.write(f'code = "{code}"\nlevel = "{a.level}"\ngroup = "{a.group}"\n'
                 'title = "TODO: a short title"\nsummary = "TODO: `what fires`: `what to write`"\n')
    for f in FILES:
        open(os.path.join(rec, f), "w", encoding="utf-8").close()
    print(f"{code} {a.name}: spec/lints/{a.name}/ scaffolded")
    print(f"add to spec/diagnostics.md: | {code} | {a.name} | TODO: fires on |")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
