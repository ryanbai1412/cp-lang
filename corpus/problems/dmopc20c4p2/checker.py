#!/usr/bin/env python3
"""Checker for DMOPC '20 Contest 4 P2: the move count equals the answer's (the minimum), and
flipping the listed in-grid cells leaves every row and column sum even."""
import sys
from collections import Counter


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, m, k = inp[:3]
    out = open(sys.argv[2]).read().split()
    want = int(open(sys.argv[3]).read().split()[0])
    if not out or not out[0].lstrip("-").isdigit():
        fail("expected the move count")
    a = int(out[0])
    if a != want:
        fail(f"A = {a}, want {want}")
    if len(out) != 1 + 2 * a:
        fail(f"expected {1 + 2 * a} tokens, got {len(out)}")
    rows, cols = Counter(), Counter()
    for i in range(k):
        rows[inp[3 + 2 * i]] ^= 1
        cols[inp[4 + 2 * i]] ^= 1
    for i in range(a):
        x, y = int(out[1 + 2 * i]), int(out[2 + 2 * i])
        if not (1 <= x <= n and 1 <= y <= m):
            fail(f"move {i + 1} ({x}, {y}) is outside the grid")
        rows[x] ^= 1
        cols[y] ^= 1
    if any(rows.values()) or any(cols.values()):
        fail("some row or column sum is odd")


main()
