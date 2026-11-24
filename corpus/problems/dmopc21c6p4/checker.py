#!/usr/bin/env python3
"""Checker for DMOPC '21 C6 P4: -1 exactly when the answer is -1; otherwise the answer's K,
one colour per road in 1..=K, no one-colour C-D path, and a one-colour A-B path as short as
the answer's."""
import sys
from collections import deque


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def one_colour_dist(n, edges, colour, k, s, t):
    best = None
    for want in range(1, k + 1):
        adj = [[] for _ in range(n + 1)]
        for (u, v), c in zip(edges, colour):
            if c == want:
                adj[u].append(v)
                adj[v].append(u)
        dist = [-1] * (n + 1)
        dist[s] = 0
        q = deque([s])
        while q:
            u = q.popleft()
            for v in adj[u]:
                if dist[v] < 0:
                    dist[v] = dist[u] + 1
                    q.append(v)
        if dist[t] >= 0 and (best is None or dist[t] < best):
            best = dist[t]
    return best


def main():
    tok = list(map(int, open(sys.argv[1]).read().split()))
    n, m = tok[0], tok[1]
    edges = [(tok[2 + 2 * i], tok[3 + 2 * i]) for i in range(m)]
    a, b, c, d = tok[2 + 2 * m: 6 + 2 * m]
    out = list(map(int, open(sys.argv[2]).read().split()))
    ans = list(map(int, open(sys.argv[3]).read().split()))
    if ans == [-1] or out == [-1]:
        if out != ans:
            fail(f"got {out[:2]}, answer {ans[:2]}")
        return
    if len(out) != m + 1 or out[0] != ans[0]:
        fail(f"want K = {ans[0]} and {m} colours")
    k, colour = out[0], out[1:]
    if any(not 1 <= x <= k for x in colour):
        fail("a colour is out of range")
    if one_colour_dist(n, edges, colour, k, c, d) is not None:
        fail("a one-colour C-D path exists")
    got = one_colour_dist(n, edges, colour, k, a, b)
    want = one_colour_dist(n, edges, ans[1:], ans[0], a, b)
    if got != want:
        fail(f"shortest one-colour A-B path {got}, answer has {want}")


main()
