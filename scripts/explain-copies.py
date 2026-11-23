#!/usr/bin/env python3
# Enforces handbook/guidelines/conformance-checklist.md#conformancer6-results-agree-across-engines-with-and-without-sanitizers
"""Check that every copy `cpl explain` lists is a copy in the generated C++.

For each `ok` conformance case, runs `cpl explain`; where it lists copies
(`[copy]` lines and `explicit copy` views), builds the judge C++ and counts
the deep copies in it: a container or string constructed from another
(`= std::vector<T>(x);`, `= std::string(x);`) and `cpl::to_list(...)`. A
case whose explain lists more copies than its C++ makes is a false claim
(spec/tools/explain.md: each heap value the judge build copies, R05.3.5).

    uv run scripts/explain-copies.py [--check] [--max N] [--jobs N]

`--max` is shrink-only: lower it as tools fixes false claims.
"""
import argparse
import concurrent.futures as cf
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
COPY = re.compile(r"=\s*(?:std::vector<[^;]*?>|std::string)\(\w+\);|cpl::to_list\(")


def one(cpl, case, tmp):
    text = case.read_text()
    if not text.startswith("// expect: ok") or "\n// tool:" in text:
        return None
    ex = subprocess.run([cpl, "explain", str(case)], capture_output=True, text=True, timeout=120).stdout
    claimed = ex.count("[copy]") + ex.count("explicit copy")
    if not claimed:
        return None
    out = Path(tmp) / (case.stem + ".cpp")
    if subprocess.run([cpl, "build", str(case), "-o", str(out)], capture_output=True, timeout=120).returncode:
        return None
    made = len(COPY.findall(out.read_text()))
    return case.relative_to(ROOT), claimed, made


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--max", type=int, default=0, help="false claims allowed (shrink-only)")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--cpl", default=str(ROOT / "target" / "debug" / "cpl"))
    a = ap.parse_args()
    cases = sorted((ROOT / "spec" / "conformance").rglob("*.cp"))
    with tempfile.TemporaryDirectory() as tmp, cf.ThreadPoolExecutor(a.jobs) as pool:
        rows = [r for r in pool.map(lambda c: one(a.cpl, c, tmp), cases) if r]
    false = [r for r in rows if r[1] > r[2]]
    print(f"explain-copies: {len(rows)} cases list copies; {len(false)} list more than the C++ makes")
    for path, claimed, made in false:
        print(f"  {path}: explain lists {claimed}, the C++ makes {made}")
    if a.check and len(false) > a.max:
        print(f"explain-copies: {len(false)} false claims, more than the {a.max} allowed")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
