#!/usr/bin/env python3
"""Checker for DMOPC '21 C9 P4: -1 exactly when the answer is -1; otherwise every coin
is removed once, each while heads, flipping the coins still in adjacent cells."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    inp = open(sys.argv[1]).read().split()
    n, m = int(inp[0]), int(inp[1])
    up = [[ch == "H" for ch in row] for row in inp[2:2 + n]]
    out = list(map(int, open(sys.argv[2]).read().split()))
    want = open(sys.argv[3]).read().split()[0]
    if want == "-1" or out[:1] == [-1]:
        if out != [-1] or want != "-1":
            fail(f"output says {out[:1]}, answer says {want}")
        return
    if len(out) != 2 * n * m:
        fail(f"expected {n * m} moves, got {len(out) // 2}")
    gone = [[False] * m for _ in range(n)]
    for k in range(n * m):
        r, c = out[2 * k] - 1, out[2 * k + 1] - 1
        if not (0 <= r < n and 0 <= c < m) or gone[r][c]:
            fail(f"move {k + 1}: ({r + 1}, {c + 1}) is not a coin on the table")
        if not up[r][c]:
            fail(f"move {k + 1}: ({r + 1}, {c + 1}) is tails")
        gone[r][c] = True
        for a, b in ((r - 1, c), (r + 1, c), (r, c - 1), (r, c + 1)):
            if 0 <= a < n and 0 <= b < m and not gone[a][b]:
                up[a][b] = not up[a][b]


main()
