#!/usr/bin/env python3
"""Checker for CCO 2017 P6 (Shifty Grid): at most 1e5 well-formed shifts that leave the grid
solved."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, m = inp[:2]
    g = [inp[2 + i * m:2 + (i + 1) * m] for i in range(n)]
    try:
        out = list(map(int, open(sys.argv[2]).read().split()))
    except ValueError:
        fail("output must be integers")
    if not out:
        fail("empty output")
    k = out[0]
    if not 0 <= k <= 100_000 or len(out) != 1 + 3 * k:
        fail(f"K = {k} with {len(out) - 1} move tokens")
    for q in range(k):
        t, i, r = out[1 + 3 * q:4 + 3 * q]
        if t == 1 and 1 <= i <= n and 0 <= r <= m:
            row = g[i - 1]
            r %= m
            g[i - 1] = row[m - r:] + row[:m - r]
        elif t == 2 and 1 <= i <= m and 0 <= r <= n:
            c = [g[x][i - 1] for x in range(n)]
            for x in range(n):
                g[(x + r) % n][i - 1] = c[x]
        else:
            fail(f"bad move {q + 1}: {t} {i} {r}")
    if any(g[i][j] != i * m + j for i in range(n) for j in range(m)):
        fail("grid is not solved")


main()
