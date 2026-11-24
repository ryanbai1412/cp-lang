#!/usr/bin/env python3
"""Lint std's `///` docs against handbook/guidelines/std-docs-checklist.md.

- R8 (#r8-one-plain-summary-sentence-first): a public fn's doc starts with
  a summary paragraph, which hover shows alone, of at most 240 characters.
- R2 (#r2-empty-ties-and-order-wherever-they-apply): a public fn over a
  `[D]T` (its first parameter) with no `empty:` label.
- R7 (#r7-the-first-example-is-the-common-case): a public fn whose first
  example passes a defaulted option while a later example passes none.
- R10 (#r10-each-module-opens-with-what-it-is-for): a std module whose
  first line is not a `//!` line.

Shrink-only: `--max` in scripts/ci.py is today's count.

    uv run python scripts/std-docs-lint.py [--max N]
"""
import argparse, glob, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LIMIT = 240


def findings():
    for path in sorted(glob.glob(os.path.join(ROOT, "std", "*.cp"))):
        text = open(path).read()
        rel = os.path.relpath(path, ROOT)
        lines = text.splitlines()
        for m in PUB_FN.finditer(text):
            line = text.count("\n", 0, m.start(1)) + 1
            summary = []
            for l in doc_above(lines, line - 1):
                if not l:
                    break
                summary.append(l)
            if not summary:
                yield f"{rel}:{line}: std-docs/R8: `{m.group(1)}` has no summary line"
            elif len(" ".join(summary)) > LIMIT:
                yield f"{rel}:{line}: std-docs/R8: `{m.group(1)}`'s summary is over {LIMIT} characters"


PUB_FN = re.compile(r"^[ \t]*(?:@\w+(?:\([^)]*\))?\s+)*pub\s+fn\s+(\w+)\s*(?:<[^>]*>)?\(", re.M)


def params(text, at):
    """(name, has_default) of the parameter list opening at `text[at - 1]`, split at depth 0."""
    depth, cur, out = 0, "", []
    for ch in text[at:]:
        if ch in "([<{":
            depth += 1
        elif ch in ")]>}":
            if depth == 0:
                break
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    out.append(cur)
    return [(head.split(":")[0].strip(), head.split(":", 1)[-1].strip(), bool(default.strip()))
            for head, _, default in (q.partition("=") for q in out) if head.strip()]


def doc_above(lines, i):
    """The `///` lines above line `i`, past its decorators."""
    j = i - 1
    while j >= 0 and lines[j].lstrip().startswith("@"):
        j -= 1
    doc = []
    while j >= 0 and lines[j].lstrip().startswith("///"):
        doc.insert(0, lines[j].strip()[3:].strip())
        j -= 1
    return doc


def label_findings(texts):
    """std-docs R2, R7 and R10 over (rel, text) pairs."""
    for rel, text in texts:
        if not text.startswith("//!"):
            yield f"{rel}:1: std-docs/R10: the module does not open with a `//!` line naming its job"
        lines = text.splitlines()
        for m in PUB_FN.finditer(text):
            line = text.count("\n", 0, m.start(1)) + 1
            name, ps = m.group(1), params(text, m.end())
            doc = doc_above(lines, line - 1)
            if ps and ps[0][0] == "self" and re.match(r"(mut |lent )?\[", ps[0][1]) \
                    and not any(d.startswith("empty:") for d in doc):
                yield f"{rel}:{line}: std-docs/R2: `{name}` reads a sequence but has no `empty:` label"
            opts = [n for n, _, default in ps if default]
            uses = [[o for o in opts if re.search(rf"\b{o}:", d)] for d in doc if d.startswith("example:")]
            if len(uses) >= 2 and uses[0] and any(not u for u in uses[1:]):
                yield (f"{rel}:{line}: std-docs/R7: `{name}`'s first example passes "
                       f"{', '.join(f'`{o}:`' for o in uses[0])}; lead with the common call")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--max", type=int, default=None)
    a = ap.parse_args()
    paths = sorted(glob.glob(os.path.join(ROOT, "std", "*.cp")))
    texts = [(os.path.relpath(p, ROOT), open(p).read()) for p in paths]
    found = sorted(set(findings()) | set(label_findings(texts)))
    for f in found:
        print(f)
    print(f"std-docs lint: {len(found)} finding(s)")
    if a.max is not None and len(found) > a.max:
        print(f"over the ratchet bound {a.max}: fix the new finding above")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
