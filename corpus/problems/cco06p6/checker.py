#!/usr/bin/env python3
"""Checker for CCC 2006 Stage 2 P6 Paint by Numbers: any n x m grid of . and * whose row and
column runs of stars are the input's clues (the statement accepts any such grid)."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def runs(cells):
    return [len(x) for x in "".join(cells).split(".") if x]


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    lines = open(sys.argv[1], encoding="utf-8").read().split("\n")
    n, m = int(lines[0]), int(lines[1])
    clues = [[int(x) for x in line.split()][:-1] for line in lines[2:2 + n + m]]
    grid = open(sys.argv[2], encoding="utf-8").read().split()
    if len(grid) != n or any(len(row) != m or set(row) - set(".*") for row in grid):
        fail(f"expected {n} lines of {m} '.' or '*'")
    for i in range(n):
        if runs(grid[i]) != clues[i]:
            fail(f"row {i + 1} has runs {runs(grid[i])}, wants {clues[i]}")
    for j in range(m):
        col = [grid[i][j] for i in range(n)]
        if runs(col) != clues[n + j]:
            fail(f"column {j + 1} has runs {runs(col)}, wants {clues[n + j]}")


if __name__ == "__main__":
    main()
