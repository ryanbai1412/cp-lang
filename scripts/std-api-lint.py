#!/usr/bin/env python3
"""Lint std's public signatures against handbook/guidelines/std-api-checklist.md.

- R4 (#r4-nothing-is-t-never-a-sentinel): a public fn returning a number
  whose doc gives a sentinel (`-1`, `INF`, "`len` when none") for "nothing".
  Orderings and signs (`cmp`, `compare`, `*_sign`), where `-1` is a value,
  are not sentinels, nor is a value the result's arithmetic uses, whose
  doc says it is "not a sentinel" (the eertree's odd-root length `-1`).
- R6 (#r6-options-are-named-arguments-with-defaults): a public fn `f_by`,
  `f_key` or `f_with` beside a public `f`: a variant where an option belongs.
- R11 (#r11-one-form-per-operation): a top-level public fn whose first
  parameter's type also has a public method of that name: both forms.

Shrink-only: `--max` in scripts/check.sh is today's count.

    uv run python scripts/std-api-lint.py [--max N]
"""
import argparse, glob, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DOC_FN = re.compile(
    r"((?:^[ \t]*///[^\n]*\n)*)[ \t]*(?:@\w+\s+)*pub\s+fn\s+(\w+)[^\n]*?"
    r"(?:->\s*([^=:\n]+?))?\s*(?:=|:\s*$)", re.M)
SENTINEL = re.compile(r"`-1`|`-?INF`|\bINF\b|`len` (?:when|if)|`n` (?:when|if) (?:none|no|not)")
ORDERING = re.compile(r"^(cmp|compare)$|_sign$")
VARIANT = re.compile(r"_(by|key|with)$")
WORDS = {"starts_with", "ends_with"}
OWNER = re.compile(r"^(?:pub\s+)?(?:struct|enum|trait|impl(?:<[^>]*>)?)\s+(?:[\w<>, ]+\s+for\s+)?(\w+)")
PUB_FN = re.compile(r"^(\s*)(?:@\w+(?:\([^)]*\))?\s+)*pub\s+fn\s+(\w+)\s*(?:<[^>]*>)?\(\s*(?:\w+\s*:\s*(?:mut\s+|lent\s+)?(\w+))?")


def both_forms(texts):
    """std-api/R11: an operation that is both a free fn and a method of its first parameter's type."""
    methods, free = {}, []
    for rel, text in texts:
        owner = None
        for line_no, line in enumerate(text.split("\n"), 1):
            o = OWNER.match(line)
            if o:
                owner = o.group(1)
                continue
            m = PUB_FN.match(line)
            if not m:
                continue
            if m.group(1):
                methods.setdefault(owner, set()).add(m.group(2))
            elif m.group(3):
                free.append((rel, line_no, m.group(2), m.group(3)))
    for rel, line_no, name, ty in free:
        if name in methods.get(ty, ()):
            yield (f"{rel}:{line_no}: std-api/R11: `{name}` is both a free fn and a method of `{ty}`; "
                   "keep one form")


def findings():
    paths = sorted(glob.glob(os.path.join(ROOT, "std", "*.cp")))
    names = {m.group(2) for p in paths for m in DOC_FN.finditer(open(p).read())}
    for path in paths:
        text = open(path).read()
        rel = os.path.relpath(path, ROOT)
        for m in DOC_FN.finditer(text):
            doc, name, ret = m.group(1), m.group(2), (m.group(3) or "").strip()
            line = text.count("\n", 0, m.start(2)) + 1
            if ret in ("Int", "I64", "F64") and not ORDERING.search(name):
                s = SENTINEL.search(doc)
                if s and "not a sentinel" not in doc:
                    yield f"{rel}:{line}: std-api/R4: `{name}` returns {s.group(0)} for nothing; return `?{ret}`"
            v = VARIANT.search(name)
            if v and name[: v.start()] in names and name not in WORDS:
                yield f"{rel}:{line}: std-api/R6: `{name}` is a variant; make the option a named argument"
    yield from both_forms((os.path.relpath(p, ROOT), open(p).read()) for p in paths)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--max", type=int, default=None)
    a = ap.parse_args()
    found = sorted(set(findings()))
    for f in found:
        print(f)
    print(f"std-api lint: {len(found)} finding(s)")
    if a.max is not None and len(found) > a.max:
        print(f"over the ratchet bound {a.max}: fix the new finding above")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
