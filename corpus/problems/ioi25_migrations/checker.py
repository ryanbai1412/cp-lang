#!/usr/bin/env python3
"""Checker for IOI 2025 Migrations, as the sample grader prints it: S[1..N-1], then U V.
Accepts when U, V are at the tree's maximum distance and S scores in full: every message
in 0..=20000, at most 8 nonzero (M <= 8) and none above 4 (Z <= 4)."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def far(adj, src):
    dist = [-1] * len(adj)
    dist[src] = 0
    order = [src]
    for x in order:
        for y in adj[x]:
            if dist[y] < 0:
                dist[y] = dist[x] + 1
                order.append(y)
    return dist


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, par = inp[0], inp[1:]
    out = open(sys.argv[2]).read().split()
    if len(out) != n + 1:
        fail(f"expected {n + 1} tokens (S[1..N-1], U, V), got {len(out)}")
    try:
        s, (u, v) = list(map(int, out[:n - 1])), map(int, out[n - 1:])
    except ValueError:
        fail("a token is not an integer")
    if any(not 0 <= x <= 20000 for x in s):
        fail("a message is outside 0..=20000")
    z, m = max(s, default=0), sum(1 for x in s if x)
    if z > 4 or m > 8:
        fail(f"scores below full: {m} messages, largest {z} (full needs M <= 8 and Z <= 4)")
    if not (0 <= u < n and 0 <= v < n):
        fail(f"(U, V) = ({u}, {v}) is not a pair of sites")
    adj = [[] for _ in range(n)]
    for i, p in enumerate(par, 1):
        adj[i].append(p)
        adj[p].append(i)
    d0 = far(adj, 0)
    a = max(range(n), key=lambda x: d0[x])
    diam = max(far(adj, a))
    dist = far(adj, u)[v]
    if dist != diam:
        fail(f"distance of (U, V) = ({u}, {v}) is {dist}, the maximum is {diam}")


if __name__ == "__main__":
    main()
