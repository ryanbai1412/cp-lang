#!/usr/bin/env python3
"""Allocate the next free lint code and scaffold its record in spec/lints/<name>/.

Usage:
    python3 scripts/new-lint.py NAME [--series 6] [--group idiom] [--level warn]
    python3 scripts/new-lint.py --next [--series 6]   # print the code only

Without --series the code comes from the first lint series (L05xx, then L06xx)
with room, so a full series never stops a new lint.

A code counts as taken when any lint.toml under spec/lints/ or
spec/checker-lints/, or a row of spec/diagnostics.md, uses it, in the working
tree or on origin/main (fetched first, best effort), so a stale checkout does
not hand out a code another lane already landed. Two allocations that both
happen before either lands can still collide; `record_codes_are_unique` in
cpl-lint catches that, so land the record soon after allocating it.

A NAME that is a row of spec/idioms/lints.md claims it: the row leaves the
catalog (a built lint lives in its record) and its family, practice and text
fill lint.toml, so a second lane starting on that row conflicts on the row
and the record directory instead of building the same lint under a new name.
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
LINT_SERIES = (5, 6)
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


def claim_row(catalog: str, name: str):
    """`(catalog without NAME's row, {family, catches, write, practice})`, or None
    when NAME isn't a row of spec/idioms/lints.md."""
    family = None
    lines = catalog.splitlines(keepends=True)
    for i, line in enumerate(lines):
        if line.startswith("### "):
            family = line[4:].strip()
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if line.startswith("|") and cells[0] == name and len(cells) >= 5:
            row = {"family": family, "catches": cells[1], "write": cells[2], "practice": cells[4]}
            return "".join(lines[:i] + lines[i + 1:]), row
    return None


def toml_str(s: str) -> str:
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def first_free(used: set[int], series=LINT_SERIES) -> int:
    """The next code in the first of `series` that isn't full."""
    for s in series:
        lo = s * 100
        if max([c for c in used if lo <= c < lo + 100], default=lo) + 1 < lo + 100:
            return next_code(used, s)
    raise SystemExit("every lint series is full")


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
    p.add_argument("--series", type=int, help="hundreds of the code: 2, 4, 5, 6 (default: the first of 5, 6 with room)")
    p.add_argument("--group", default="idiom", help="correctness, perf, idiom, style or pedantic")
    p.add_argument("--rule", help="the checklist rule it enforces, e.g. idiomatic-code/R3")
    p.add_argument("--level", default="warn", help="allow, warn or deny")
    p.add_argument("--next", action="store_true", help="print the next free code and stop")
    p.add_argument("--no-fetch", action="store_true", help="don't fetch origin/main first")
    a = p.parse_args(argv)
    used = used_codes(not a.no_fetch)
    code = f"L{(first_free(used) if a.series is None else next_code(used, a.series)):04d}"
    if a.next:
        print(code)
        return 0
    if not a.name or not NAME_RE.match(a.name):
        print("error: give a snake_case NAME", file=sys.stderr)
        return 2
    if not a.rule:
        print("error: give --rule, the checklist rule it enforces", file=sys.stderr)
        return 2
    rec = os.path.join(ROOT, "spec/lints", a.name)
    if os.path.exists(rec):
        print(f"error: {os.path.relpath(rec, ROOT)} already exists", file=sys.stderr)
        return 1
    catalog_path = os.path.join(ROOT, "spec/idioms/lints.md")
    with open(catalog_path, encoding="utf-8") as fh:
        claimed = claim_row(fh.read(), a.name)
    summary = '"<`what fires`: `what to write`>"'
    extra = ""
    if claimed:
        catalog, row = claimed
        with open(catalog_path, "w", encoding="utf-8") as fh:
            fh.write(catalog)
        summary = toml_str(f"{row['catches']}: {row['write']}")
        extra = f"family = {toml_str(row['family'] or '')}\npractice = {toml_str(row['practice'])}\n"
    os.makedirs(rec)
    with open(os.path.join(rec, "lint.toml"), "w", encoding="utf-8") as fh:
        fh.write(f'code = "{code}"\nlevel = "{a.level}"\ngroup = "{a.group}"\n'
                 'title = "<a short title>"\n'
                 f'rule = "{a.rule}"\n'
                 f'summary = {summary}\n{extra}')
    for f in FILES:
        open(os.path.join(rec, f), "w", encoding="utf-8").close()
    print(f"{code} {a.name}: spec/lints/{a.name}/ scaffolded")
    if claimed:
        print(f"claimed its spec/idioms/lints.md row ({row['family']}); land the record with the row's removal")
    else:
        print("not a spec/idioms/lints.md row: a new lint needs that file's 'Adding a lint' evidence")
    print(f"add to spec/diagnostics.md: | {code} | {a.name} | <fires on> |")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
