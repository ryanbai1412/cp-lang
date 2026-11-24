#!/usr/bin/env python3
"""Checker for DMOJ piggy: any drawing is accepted (the beauty contest is the judges' call), so
accept 1..=1000 lines of at most 1000 characters each with at least one visible character."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    rows = open(sys.argv[2], encoding="utf-8", errors="replace").read().rstrip("\n").split("\n")
    if not any(row.strip() for row in rows):
        fail("the drawing is empty")
    if len(rows) > 1000:
        fail(f"{len(rows)} rows, at most 1000 allowed")
    wide = max(len(row) for row in rows)
    if wide > 1000:
        fail(f"a row of {wide} characters, at most 1000 allowed")


if __name__ == "__main__":
    main()
