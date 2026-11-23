#!/usr/bin/env python3
"""The trait matrix of std containers, for std-api/R12.

handbook/guidelines/std-api-checklist.md#r12-every-container-meets-the-common-traits:
every container prints, compares, walks and has `len` and `is_empty`. A
container is a `pub struct` in std/*.cp with `pub fn len(self) -> Int`.

- `Show`: an `impl Show for T` (core 10-io), so `print(x)` and traces work.
- walk: `each`, `iter` or `items` yielding its elements in a defined order.
- `is_empty` beside `len`.
- `Eq` is structural for every struct (core 12-powers), so it isn't listed.

A struct whose doc says `not a container: <why>` is skipped. Shrink-only:
`--max` in scripts/check.sh is today's count of missing (type, trait) pairs.

    uv run python scripts/std-trait-matrix.py [--max N]
"""
import argparse, glob, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STRUCT = re.compile(r"((?:^///[^\n]*\n)*)^pub struct (\w+)(?:<[^>]*>)?[^\n]*\n((?:(?:    [^\n]*)?\n)*)", re.M)


def matrix():
    for path in sorted(glob.glob(os.path.join(ROOT, "std", "*.cp"))):
        text = open(path).read()
        rel = os.path.relpath(path, ROOT)
        for m in STRUCT.finditer(text):
            doc, name, body = m.groups()
            if not re.search(r"pub fn len\(self\) -> Int\b", body) or "not a container:" in doc:
                continue
            line = text.count("\n", 0, m.start(2)) + 1
            missing = []
            if not re.search(rf"^impl(?:<[^>]*>)? Show for {name}\b", text, re.M):
                missing.append("Show")
            if not re.search(r"pub fn (?:each|iter|items)\(self\)[^\n]*yields", body):
                missing.append("walk")
            if not re.search(r"pub fn is_empty\(self\)", body):
                missing.append("is_empty")
            yield rel, line, name, missing


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--max", type=int, default=None)
    a = ap.parse_args()
    gaps = 0
    rows = list(matrix())
    for rel, line, name, missing in rows:
        for trait in missing:
            print(f"{rel}:{line}: std-api/R12: `{name}` has no {trait}")
        gaps += len(missing)
    print(f"std trait matrix: {len(rows)} containers, {gaps} missing (type, trait) pairs")
    if a.max is not None and gaps > a.max:
        print(f"over the ratchet bound {a.max}: add the trait above, or say `not a container: <why>` in its doc")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
