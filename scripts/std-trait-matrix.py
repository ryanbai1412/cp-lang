#!/usr/bin/env python3
"""The trait matrix of std containers, for std-api/R12.

handbook/guidelines/std-api-checklist.md#r12-every-container-meets-the-common-traits:
every container prints, compares, walks and has `len` and `is_empty`. A
container is a `pub struct` in std/*.cp with `pub fn len(self) -> Int`.

- `Show`: an `impl Show for T` (core 10-io), so `print(x)` and traces work.
- walk: an `impl Seq`, or `each`, `iter` or `items` yielding its elements in a defined order.
- `is_empty` beside `len`.
- `Eq` is structural for every struct (core 12-powers), so it isn't listed.

A struct whose doc says `not a container: <why>` is skipped. A type whose
nodes live in an arena it is passed (`PMultiset` in a `PArena`) walks and
prints through `x.within(ar) -> lends V`: `V` is checked as a container, so
the type itself needs only `is_empty`. Hard zero: `--max 0` in scripts/ci.py
and `std_trait_matrix` in scripts/debt-ratchet.py.

    uv run python scripts/std-trait-matrix.py [--max N]
"""
import argparse, glob, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STRUCT = re.compile(r"((?:^///[^\n]*\n)*)^pub struct (\w+)(?:<[^>]*>)?[^\n]*\n((?:(?:    [^\n]*)?\n)*)", re.M)


def matrix(root=ROOT):
    paths = sorted(glob.glob(os.path.join(root, "std", "*.cp")))
    every = "\n".join(open(p).read() for p in paths)
    for path in paths:
        text = open(path).read()
        rel = os.path.relpath(path, root)
        for m in STRUCT.finditer(text):
            doc, name, body = m.groups()
            if not re.search(r"pub fn len\(self\) -> Int\b", body) or "not a container:" in doc:
                continue
            line = text.count("\n", 0, m.start(2)) + 1
            missing = []
            view = re.search(r"pub fn within\(self, [^)]*\) -> lends (\w+)", body)
            viewed = view and re.search(rf"^pub struct {view.group(1)}\b", text, re.M)
            if not viewed and not re.search(rf"^impl(?:<(?:[^<>\n]|<[^<>\n]*>)*>)? Show for {name}\b", every, re.M):
                missing.append("Show")
            walks = re.search(r"pub fn (?:each|iter|items)(?:<[^>]*>)?\(self\)[^\n]*yields", body) or re.search(
                rf"^impl\b[^\n]* Seq<[^\n]*> for {name}\b", text, re.M)
            if not viewed and not walks:
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
