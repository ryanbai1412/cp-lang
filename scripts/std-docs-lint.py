#!/usr/bin/env python3
"""Lint std's `///` docs against handbook/guidelines/std-docs-checklist.md.

- R8 (#r8-one-plain-summary-sentence-first): a public fn's doc starts with
  a summary paragraph, which hover shows alone, of at most 240 characters.

Shrink-only: `--max` in scripts/check.sh is today's count.

    uv run python scripts/std-docs-lint.py [--max N]
"""
import argparse, glob, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DOC_FN = re.compile(r"((?:^[ \t]*///[^\n]*\n)*)[ \t]*(?:@\w+\s+)*pub\s+fn\s+(\w+)", re.M)
LIMIT = 240


def findings():
    for path in sorted(glob.glob(os.path.join(ROOT, "std", "*.cp"))):
        text = open(path).read()
        rel = os.path.relpath(path, ROOT)
        for m in DOC_FN.finditer(text):
            line = text.count("\n", 0, m.start(2)) + 1
            lines = [l.strip()[3:].strip() for l in m.group(1).splitlines()]
            summary = []
            for l in lines:
                if not l:
                    break
                summary.append(l)
            if not summary:
                yield f"{rel}:{line}: std-docs/R8: `{m.group(2)}` has no summary line"
            elif len(" ".join(summary)) > LIMIT:
                yield f"{rel}:{line}: std-docs/R8: `{m.group(2)}`'s summary is over {LIMIT} characters"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--max", type=int, default=None)
    a = ap.parse_args()
    found = sorted(set(findings()))
    for f in found:
        print(f)
    print(f"std-docs lint: {len(found)} finding(s)")
    if a.max is not None and len(found) > a.max:
        print(f"over the ratchet bound {a.max}: fix the new finding above")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
