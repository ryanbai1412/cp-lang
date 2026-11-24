#!/usr/bin/env python3
"""Checker for yac6p1: M distinct friendships among 1..N. With M >= N - 1 the least
sum of separations is M * 1 + (C(N,2) - M) * 2, reached exactly when every pair is at
distance <= 2, so the network must be connected with diameter at most 2."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    n, m = map(int, open(sys.argv[1], encoding="utf-8").read().split())
    tok = open(sys.argv[2], encoding="utf-8").read().split()
    if len(tok) != 2 * m:
        fail(f"expected {m} pairs, got {len(tok)} numbers")
    try:
        vals = list(map(int, tok))
    except ValueError:
        fail("non-integer token")
    adj = [0] * (n + 1)
    seen = set()
    for k in range(m):
        u, v = vals[2 * k], vals[2 * k + 1]
        if not (1 <= u <= n and 1 <= v <= n and u != v):
            fail(f"pair {k + 1} ({u} {v}) is not two different people in 1..{n}")
        key = (min(u, v), max(u, v))
        if key in seen:
            fail(f"friendship {key} listed twice")
        seen.add(key)
        adj[u] |= 1 << v
        adj[v] |= 1 << u
    full = (1 << (n + 1)) - 2
    for u in range(1, n + 1):
        reach = adj[u] | (1 << u)
        nb = adj[u]
        while nb:
            low = nb & -nb
            reach |= adj[low.bit_length() - 1]
            nb ^= low
        if reach != full:
            missing = (full & ~reach).bit_length() - 1
            fail(f"people {u} and {missing} are more than 2 apart")


if __name__ == "__main__":
    main()
