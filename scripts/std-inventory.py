#!/usr/bin/env python3
"""Every row of spec/lib/inventory.md spells its operation with names std documents.

Enforces handbook/guidelines/std-api-checklist.md#r15 (comparative inventory),
checked against `cpl doc --format json std`.

  scripts/std-inventory.py [--cpl PATH]
"""
import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TABLE = ROOT / "spec/lib/inventory.md"
ROW = re.compile(r"^\| *(C\+\+|Python|Rust|ACL) *\|(.+)\|(.*)\| *$")
NAME = re.compile(r"\.([a-z_]\w*)|\b([a-z_]\w*)\(|\b([A-Z]\w*)")


def rows(text):
    for n, line in enumerate(text.splitlines(), 1):
        m = ROW.match(line)
        if m:
            yield n, m.group(1), m.group(2).strip(), m.group(3).strip()


def spelled(cell):
    names = set()
    for span in re.findall(r"`([^`]+)`", cell):
        for m in NAME.finditer(span):
            names.add(next(g for g in m.groups() if g))
    return names


def documented(doc):
    names = set()

    def walk(items):
        for item in items:
            names.add(item["name"].split("<")[0])
            walk(item.get("members") or [])

    for module in doc["modules"]:
        walk(module["items"])
    return names


def findings(text, known):
    out = []
    for n, lib, op, cell in rows(text):
        names = spelled(cell)
        if not names:
            out.append(f"{TABLE.name}:{n}: {lib} {op} has no CP Lang spelling")
        for name in sorted(names - known):
            out.append(f"{TABLE.name}:{n}: {lib} {op}: `{name}` is not a documented std name")
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cpl", default=str(ROOT / "target/release/cpl"))
    args = ap.parse_args()
    env = dict(os.environ, CPL_STD_DIR=str(ROOT / "std"))
    doc = subprocess.run([args.cpl, "doc", "--format", "json", str(ROOT / "std")],
                         capture_output=True, text=True, env=env, check=True)
    text = TABLE.read_text()
    out = findings(text, documented(json.loads(doc.stdout)))
    print("\n".join(out + [f"std inventory: {sum(1 for _ in rows(text))} rows, {len(out)} finding(s)"]))
    return 1 if out else 0


if __name__ == "__main__":
    sys.exit(main())
