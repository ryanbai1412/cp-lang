#!/usr/bin/env python3
"""Checker for BSSPC '21 S1: -1 exactly when the reference says -1; otherwise N rows of M
lowercase letters that keep every given letter and make every row and column a palindrome."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    head, *grid = open(sys.argv[1]).read().split()
    n, m = int(head), int(grid.pop(0))
    out = open(sys.argv[2]).read().split()
    ref = open(sys.argv[3]).read().split()
    if ref == ["-1"] or out == ["-1"]:
        if out != ref:
            fail(f"got {' '.join(out[:1])}, want {ref[0]}")
        return
    if len(out) != n or any(len(row) != m for row in out):
        fail(f"expected {n} rows of {m} letters")
    for i in range(n):
        for j in range(m):
            c = out[i][j]
            if not ("a" <= c <= "z"):
                fail(f"({i + 1}, {j + 1}) is {c!r}, not a lowercase letter")
            if grid[i][j] != "." and grid[i][j] != c:
                fail(f"({i + 1}, {j + 1}) was given as {grid[i][j]}")
            if c != out[n - 1 - i][j] or c != out[i][m - 1 - j]:
                fail(f"row {i + 1} or column {j + 1} is not a palindrome")


main()
