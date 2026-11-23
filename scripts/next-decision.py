#!/usr/bin/env python3
"""Print the next unused decision number in decisions.md, or --check for duplicates.

Usage:
    python3 scripts/next-decision.py          # print next integer (max + 1 over this tree and origin/main)
    python3 scripts/next-decision.py --check  # exit 1 if duplicate heading numbers exist
"""
import argparse
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DECISIONS = os.path.join(ROOT, "decisions.md")
HEADING_RE = re.compile(r"^##\s+(\d+)\.", re.MULTILINE)


def parse_decision_numbers(text: str) -> list[int]:
    return [int(m.group(1)) for m in HEADING_RE.finditer(text)]


def next_number(text: str) -> int:
    nums = parse_decision_numbers(text)
    return (max(nums) + 1) if nums else 1


def upstream_text(ref: str = "origin/main") -> str:
    """decisions.md on `ref`, so a number another lane just landed is not reused."""
    subprocess.run(["git", "fetch", "-q", "origin", "main"], cwd=ROOT, capture_output=True)
    r = subprocess.run(["git", "show", f"{ref}:decisions.md"], cwd=ROOT, capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else ""


def find_duplicates(text: str) -> list[int]:
    seen: set[int] = set()
    dups: list[int] = []
    for n in parse_decision_numbers(text):
        if n in seen and n not in dups:
            dups.append(n)
        seen.add(n)
    return dups


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--file", default=DECISIONS, help="Path to decisions.md")
    parser.add_argument(
        "--check", action="store_true", help="Fail if duplicate decision numbers exist"
    )
    args = parser.parse_args(argv)
    with open(args.file, encoding="utf-8") as fh:
        text = fh.read()
    if args.check:
        dups = find_duplicates(text)
        if dups:
            print(f"duplicate decision numbers in {args.file}: {dups}", file=sys.stderr)
            return 1
        print(f"ok ({len(parse_decision_numbers(text))} decisions, next is {next_number(text)})")
        return 0
    print(max(next_number(text), next_number(upstream_text())))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
