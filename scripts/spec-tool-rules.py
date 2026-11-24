#!/usr/bin/env python3
# Enforces handbook/guidelines/spec-rules-checklist.md#spec-rulesr8-every-tool-feature-has-a-rule-id
"""Count the tool features in spec/tools/ that carry no rule id.

A feature is a top-level bullet (`- ...`, with its continuation lines). It
has a rule id when an id ends one of its sentences: `(Rnn.n.n)` as in
spec/core, or a tool id such as `(Rtrace.4)`, which cases cite as `trace.4`. Without one, no case can cite it and the coverage gate can't
see it. `--max` is shrink-only.

    uv run scripts/spec-tool-rules.py [--max N] [--list]
"""
import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RULE = re.compile(r"\(R(?:\d\d[a-z]?|[a-z]+)(?:\.\d+)+(?:, permits)?\)")


def features(path):
    item, out, fence = None, [], False
    for i, line in enumerate(path.read_text().splitlines(), 1):
        if line.startswith("```"):
            fence = not fence
        if fence:
            continue
        if line.startswith("- "):
            if item:
                out.append(item)
            item = [i, line]
        elif item and line.startswith("  ") and line.strip():
            item[1] += " " + line.strip()
        elif item:
            out.append(item)
            item = None
    if item:
        out.append(item)
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--max", type=int)
    ap.add_argument("--list", action="store_true")
    a = ap.parse_args()
    bare = []
    total = 0
    for f in sorted((ROOT / "spec" / "tools").glob("*.md")):
        for line, text in features(f):
            total += 1
            if not RULE.search(text):
                bare.append((f.relative_to(ROOT), line, text))
    print(f"spec-tool-rules: {len(bare)} of {total} tool features have no rule id")
    if a.list:
        for path, line, text in bare:
            print(f"  {path}:{line}: {text[:90]}")
    if a.max is not None and len(bare) > a.max:
        print(f"spec-tool-rules: more than the {a.max} allowed; give the new feature a rule id and a case")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
