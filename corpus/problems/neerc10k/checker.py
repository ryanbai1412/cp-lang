#!/usr/bin/env python3
"""Checker for NEERC 2010 K: k is the least odd number >= the maximum degree, then a colour
in 1..=k per vertex with no edge joining two equal colours."""
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
    edges = [(inp[2 + 2 * i], inp[3 + 2 * i]) for i in range(m)]
    deg = [0] * (n + 1)
    for u, v in edges:
        deg[u] += 1
        deg[v] += 1
    want = max(deg[1:]) | 1
    out = open(sys.argv[2]).read().split()
    if len(out) != 1 + n:
        fail(f"expected {1 + n} tokens, got {len(out)}")
    k, colour = int(out[0]), [0] + [int(c) for c in out[1:]]
    if k != want:
        fail(f"k = {k}, want {want}")
    if any(not 1 <= c <= k for c in colour[1:]):
        fail("a colour is outside 1..=k")
    for u, v in edges:
        if colour[u] == colour[v]:
            fail(f"edge {u} {v} joins two vertices of colour {colour[u]}")


main()
