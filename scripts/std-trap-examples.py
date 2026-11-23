#!/usr/bin/env python3
"""Every std `trap:` doc example traps (handbook/guidelines/std-docs-checklist.md#r4).

A `trap:` line under a pub fn's doc is a one-line program, `setup; ...; expr`,
that must stop with a runtime trap (an `R` error), not print a value and not
fail to compile. `--audit` lists the pub fns whose `traps:` names a condition
but that show no `trap:` example yet, the gap this check still has.

    uv run python scripts/std-trap-examples.py [--cpl target/debug/cpl] [--audit]
"""
import argparse, glob, os, re, subprocess, sys, tempfile

PUB_FN = re.compile(r"^\s*(?:pub\s+(?:@\w+\s+)*|@get\s+pub\s+)fn\s+(\w+)")
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def split_top(s, sep):
    """`s` split at `sep` outside brackets and strings."""
    parts, depth, quote, start, i = [], 0, None, 0, 0
    while i < len(s):
        c = s[i]
        if quote:
            if c == "\\":
                i += 1
            elif c == quote:
                quote = None
        elif c in "\"'":
            quote = c
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
        elif depth == 0 and s.startswith(sep, i):
            parts.append(s[start:i])
            start = i + len(sep)
        i += 1
    parts.append(s[start:])
    return [p.strip() for p in parts if p.strip()]


def documented(path):
    """(line, fn name, doc lines) for every pub fn in a std file."""
    lines = open(path).read().splitlines()
    for i, line in enumerate(lines):
        m = PUB_FN.match(line)
        if not m:
            continue
        doc, j = [], i - 1
        while j >= 0 and lines[j].strip().startswith("///"):
            doc.insert(0, lines[j].strip()[3:].strip())
            j -= 1
        yield i + 1, m.group(1), doc


def programs(module, example):
    """The example as a program: its last expression printed, else run as a statement."""
    stmts = split_top(example.strip().strip("`"), ";")
    last = stmts.pop()
    for tail in (f"print({last})", last):
        yield "\n".join([f"use std.{module}.*", *stmts, tail]) + "\n"


def run_trap(cpl, src):
    """`(trapped, what happened)` for one program."""
    r = subprocess.run([cpl, "run", src], capture_output=True, text=True, timeout=60)
    if r.returncode != 0 and re.search(r"^error\[R\d+\]", r.stderr, re.M):
        return True, ""
    return False, r.stdout.strip() or (r.stderr.strip().splitlines() or ["nothing"])[0]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cpl", default=os.path.join(ROOT, "target", "debug", "cpl"))
    ap.add_argument("--audit", action="store_true")
    args = ap.parse_args()
    bad, ran, uncovered = [], 0, []
    with tempfile.TemporaryDirectory() as tmp:
        for path in sorted(glob.glob(os.path.join(ROOT, "std", "*.cp"))):
            module = os.path.basename(path)[:-3]
            rel = os.path.relpath(path, ROOT)
            for line, name, doc in documented(path):
                traps = [d for d in doc if d.startswith("traps:")]
                examples = [d.split(":", 1)[1] for d in doc if d.startswith("trap:")]
                if traps and not examples and not any("never" in t for t in traps):
                    uncovered.append(f"{rel}:{line} {name}")
                for ex in examples:
                    ran += 1
                    src = os.path.join(tmp, f"{module}_{line}.cp")
                    seen = []
                    for text in programs(module, ex):
                        open(src, "w").write(text)
                        trapped, got = run_trap(args.cpl, src)
                        if trapped:
                            break
                        seen.append(got)
                    else:
                        bad.append(f"{rel}:{line} {name}: {ex.strip()} did not trap ({seen[0]})")
    for b in bad:
        print(b)
    if args.audit:
        for u in uncovered:
            print(f"no trap: example: {u}")
    print(f"std trap examples: {ran} run, {len(bad)} did not trap; {len(uncovered)} fns name a trap with no example")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
