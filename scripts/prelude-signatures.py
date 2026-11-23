#!/usr/bin/env python3
"""Every std type a prelude fn returns is a prelude name (handbook/guidelines/std-api-checklist.md#r18).

A program that gets a value from a prelude fn can name its type without a
`use`: `let w: RingSlice<Int> = a.windows(3)` works because `windows` is in
the prelude, so `RingSlice` must be too.

    uv run python scripts/prelude-signatures.py
"""
import glob, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
USE = re.compile(r"use std\.(\w+)\.\{([^}]*)\}", re.S)
TYPE = re.compile(r"^pub (?:struct|enum|trait|type) (\w+)", re.M)


def main():
    prelude = open(os.path.join(ROOT, "std", "prelude.cp")).read()
    names, by_module = set(), {}
    for m in USE.finditer(prelude):
        for n in re.split(r"[,\s]+", m.group(2)):
            if n:
                names.add(n)
                by_module.setdefault(m.group(1), []).append(n)
    types = {}
    for path in glob.glob(os.path.join(ROOT, "std", "*.cp")):
        for m in TYPE.finditer(open(path).read()):
            types[m.group(1)] = os.path.basename(path)[:-3]
    bad = set()
    for module, fns in by_module.items():
        src = open(os.path.join(ROOT, "std", f"{module}.cp")).read()
        for fn in fns:
            sig = re.compile(
                r"^\s*(?:pub\s+(?:@\w+\s+)*|@get\s+pub\s+)fn\s+" + re.escape(fn)
                + r"\b[^\n]*?->\s*([^=\n]*?)(?:\s=\s|:\s*$)", re.M)
            for m in sig.finditer(src):
                for t in re.findall(r"\b([A-Z]\w*)\b", m.group(1)):
                    if t in types and t not in names:
                        bad.add(f"std/prelude.cp: `{fn}` (std.{module}) returns `{t}`; add `{t}` from std.{types[t]} to the prelude")
    for b in sorted(bad):
        print(b)
    print(f"prelude signatures: {len(names)} names, {len(bad)} returned types missing")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
