#!/usr/bin/env python3
"""Checker for CCC 2019 S3: a 3 x 3 grid of integers in [-1e9, 1e9] that keeps every given
cell and makes each row and column an arithmetic sequence."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    given = open(sys.argv[1]).read().split()
    out = open(sys.argv[2]).read().split()
    if len(out) != 9:
        fail(f"expected 9 values, got {len(out)}")
    try:
        g = [int(w) for w in out]
    except ValueError:
        fail("non-integer value")
    for k in range(9):
        if given[k] != "X" and int(given[k]) != g[k]:
            fail(f"cell {k} was given as {given[k]}, got {g[k]}")
        if abs(g[k]) > 10 ** 9:
            fail(f"cell {k} is {g[k]}, outside [-1e9, 1e9]")
    lines = [[3 * i + j for j in range(3)] for i in range(3)] + [[3 * i + j for i in range(3)] for j in range(3)]
    for a, b, c in lines:
        if g[b] - g[a] != g[c] - g[b]:
            fail(f"cells {a}, {b}, {c} are not arithmetic")


if __name__ == "__main__":
    main()
