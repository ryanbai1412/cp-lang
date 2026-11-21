#!/usr/bin/env python3
"""Codemod for non-canonical spellings in active and dream CP Lang ports.

Canonical forms (decisions.md §20 and §53):
  - multitest divider: `@!multitest` (rewrites `@multitest`, `--- multitest ---`,
    `--- with multitest() ---`)
  - sequence folds/sorts: method form via `cpl lint --fix` (L0509/L0510)

Usage:
    python3 scripts/fix-spelling-drift.py --check   # exit 1 if any file would change
    python3 scripts/fix-spelling-drift.py --write   # rewrite files in place
"""
import argparse
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PORTS_ROOT = os.path.join(ROOT, "proto", "ports")

RETIRED_PREFIXES = (
    "scout_batch_",
    "blind_critic_",
    "hillclimb_",
    "sim3",
    "before",
    "wave1",
    "wave2",
)

NON_CANON_MULTITEST_RE = re.compile(
    r"(?m)^([ \t]*)(?:@multitest(?![ \t]*fn\b)|---[ \t]*(?:with[ \t]+)?multitest(?:\(\))?[ \t]*---)[ \t]*$"
)


def is_retired(rel_path):
    parts = rel_path.replace("\\", "/").split("/")
    return any(
        part == pref or part.startswith(pref)
        for part in parts
        for pref in RETIRED_PREFIXES
    )


def rewrite_source(src):
    return NON_CANON_MULTITEST_RE.sub(r"\1@!multitest", src)


def run_codemod(ports_root=PORTS_ROOT, write=False):
    changed = []
    for dirpath, _dirnames, filenames in os.walk(ports_root):
        for fn in sorted(filenames):
            if not fn.endswith(".cp"):
                continue
            full = os.path.join(dirpath, fn)
            rel = os.path.relpath(full, ports_root)
            if is_retired(rel):
                continue
            with open(full, encoding="utf-8", errors="replace") as f:
                src = f.read()
            new_src = rewrite_source(src)
            if new_src != src:
                changed.append(rel)
                if write:
                    with open(full, "w", encoding="utf-8") as f:
                        f.write(new_src)
    return changed


def main(argv=None):
    ap = argparse.ArgumentParser(description="Fix non-canonical spellings in active/dream ports")
    ap.add_argument("--write", action="store_true", help="rewrite files in place")
    ap.add_argument("--check", action="store_true", help="exit 1 if any file has drift")
    ap.add_argument("--ports-root", default=PORTS_ROOT, help="ports root directory")
    args = ap.parse_args(argv)

    changed = run_codemod(args.ports_root, write=args.write)
    action = "rewrote" if args.write else "would rewrite"
    print(f"fix-spelling-drift: {action} {len(changed)} file(s)")
    for rel in changed:
        print(f"  {rel}")
    if args.check and changed:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
