#!/usr/bin/env python3
# Enforces handbook/guidelines/spec-rules-checklist.md#spec-rulesr6-the-handbook-and-the-spec-change-together
"""Count decisions cited in spec/ that no handbook doc explains.

A rule in spec/ that cites `decision N` rests on a design call whose reason
and losing alternatives a reader should find in the handbook; a doc that
explains it names the decision. The count only goes down: `--max` is the
ratchet bound set in scripts/check.sh.

    uv run scripts/spec-decisions-explained.py [--max N]
"""
import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CITE = re.compile(r"[Dd]ecisions? (\d{2,4})")


def cited(paths) -> dict[str, set[str]]:
    out = {}
    for p in paths:
        for n in CITE.findall(p.read_text()):
            out.setdefault(n, set()).add(str(p.relative_to(ROOT)))
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--max", type=int)
    a = ap.parse_args()
    spec = cited((ROOT / "spec").rglob("*.md"))
    hb = cited((ROOT / "handbook").rglob("*.md"))
    gaps = sorted(set(spec) - set(hb), key=int)
    for n in gaps:
        print(f"decision {n}: cited in {', '.join(sorted(spec[n]))}; no handbook doc explains it")
    print(f"{len(gaps)} of {len(spec)} decisions cited in spec/ unexplained in handbook/")
    if a.max is not None and len(gaps) > a.max:
        print(f"over the ratchet bound {a.max}: explain the decision in handbook/language/")
        return 1
    if a.max is not None and len(gaps) < a.max:
        print(f"under the bound: lower --max in scripts/check.sh to {len(gaps)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
