#!/usr/bin/env python3
"""Checker for YAC 7 P2: -1 exactly when a side is 1; otherwise an n x m grid holding 1..=n*m
once each, every cell coprime with at least two side-adjacent cells."""
import sys
from math import gcd


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    out = open(sys.argv[2]).read().split()
    pos = 0
    for case in range(inp[0]):
        n, m = inp[1 + 2 * case], inp[2 + 2 * case]
        if n == 1 or m == 1:
            if out[pos:pos + 1] != ["-1"]:
                fail(f"test {case + 1}: {n} x {m} needs -1")
            pos += 1
            continue
        cells = out[pos:pos + n * m]
        pos += n * m
        if len(cells) != n * m or not all(c.lstrip("-").isdigit() for c in cells):
            fail(f"test {case + 1}: expected {n * m} numbers")
        g = [list(map(int, cells[i * m:(i + 1) * m])) for i in range(n)]
        if sorted(sum(g, [])) != list(range(1, n * m + 1)):
            fail(f"test {case + 1}: not a permutation of 1..={n * m}")
        for i in range(n):
            for j in range(m):
                near = [g[i + di][j + dj] for di, dj in ((1, 0), (-1, 0), (0, 1), (0, -1))
                        if 0 <= i + di < n and 0 <= j + dj < m]
                if sum(gcd(g[i][j], v) == 1 for v in near) < 2:
                    fail(f"test {case + 1}: cell ({i + 1}, {j + 1}) = {g[i][j]} has under 2 coprime neighbours")
    if pos != len(out):
        fail(f"{len(out) - pos} extra tokens")


main()
