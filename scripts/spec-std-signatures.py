#!/usr/bin/env python3
# Enforces handbook/guidelines/spec-rules-checklist.md#spec-rulesr5-chapters-and-std-say-the-same-thing
"""Check that spec/lib chapters and std give a top-level fn the same return type.

A chapter's fenced `fn name(...) -> R` line and std's `pub fn name(...) -> R`
must agree after whitespace is removed: a chapter that says `?Int` where std
returns `?K`, or `List<Int>` where std returns `List<K>`, teaches a type the
code doesn't have. Only top-level fns are compared (methods inside a struct
or trait block are indented); a name std declares more than once matches any
of its return types. Fns std doesn't declare are `scripts/conformance-prelude.py`'s
and `scripts/spec-stub.py`'s business.

    uv run scripts/spec-std-signatures.py [--check]
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HEAD = re.compile(r"^(?:pub\s+)?fn\s+(\w+)\s*")


def returns(line: str) -> tuple[str, str] | None:
    """(name, return type) of a top-level `fn` line, or None."""
    m = HEAD.match(line)
    if not m:
        return None
    i, depth = m.end(), 0
    if line[i:i + 1] == "<":
        for j in range(i, len(line)):
            depth += {"<": 1, ">": -1}.get(line[j], 0)
            if depth == 0:
                i = j + 1
                break
    if line[i:i + 1] != "(":
        return None
    for j in range(i, len(line)):
        depth += {"(": 1, ")": -1}.get(line[j], 0)
        if depth == 0:
            rest = line[j + 1:]
            break
    else:
        return None
    m2 = re.match(r"\s*->\s*(.*)", rest)
    if not m2:
        return None
    r = re.split(r"//|\s=\s|\s=$", m2.group(1))[0]
    r = re.sub(r"\s+", "", r)
    return m.group(1), r[:-1] if r.endswith(":") else r


def main() -> int:
    std: dict[str, set[str]] = {}
    for p in (ROOT / "std").glob("*.cp"):
        for line in p.read_text().split("\n"):
            if line.startswith("pub fn") and (got := returns(line)):
                std.setdefault(got[0], set()).add(got[1])
    bad = 0
    for p in sorted((ROOT / "spec" / "lib").glob("*.md")):
        in_block = False
        for n, line in enumerate(p.read_text().split("\n"), 1):
            if line.startswith("```"):
                in_block = not in_block
                continue
            got = returns(line) if in_block else None
            if got and got[0] in std and got[1] not in std[got[0]]:
                bad += 1
                print(f"{p.relative_to(ROOT)}:{n}: `{got[0]}` returns `{got[1]}` here, "
                      f"`{' | '.join(sorted(std[got[0]]))}` in std")
    print(f"{bad} chapter signatures disagreeing with std on the return type")
    return 1 if bad and "--check" in sys.argv else 0


if __name__ == "__main__":
    sys.exit(main())
