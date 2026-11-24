#!/usr/bin/env python3
"""Checker for SAC '22 CC4 Junior P5 (output-only): the printed graph meets the bounds
(1 <= N <= 1e5, M <= min(2e5, N(N-1)/2), 1 <= u != v <= N, 1 <= w <= 1e9, no duplicate
edges) and drives the statement's Dijkstra without the stale-entry skip past 1e7 edge
scans. The simulation stops at the pop whose scan would cross 1e7."""
import heapq
import sys

LIMIT = 10**7


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    tok = open(sys.argv[2], encoding="utf-8").read().split()
    try:
        vals = list(map(int, tok))
    except ValueError:
        fail("non-integer token")
    if len(vals) < 2:
        fail("missing N M")
    n, m = vals[0], vals[1]
    if not 1 <= n <= 10**5 or not 0 <= m <= min(2 * 10**5, n * (n - 1) // 2):
        fail(f"N = {n}, M = {m} out of bounds")
    if len(vals) != 2 + 3 * m:
        fail(f"expected {m} edges, got {len(vals) - 2} numbers")
    adj = [[] for _ in range(n + 1)]
    seen = set()
    for k in range(m):
        u, v, w = vals[2 + 3 * k: 5 + 3 * k]
        if not (1 <= u <= n and 1 <= v <= n and u != v and 1 <= w <= 10**9):
            fail(f"edge {k + 1} ({u} {v} {w}) out of bounds")
        key = (min(u, v), max(u, v))
        if key in seen:
            fail(f"duplicate edge {key}")
        seen.add(key)
        adj[u].append((v, w))
        adj[v].append((u, w))
    d = [None] * (n + 1)
    d[1] = 0
    pq = [(0, 1)]
    counter = 0
    while pq:
        _, v = heapq.heappop(pq)
        if counter + len(adj[v]) > LIMIT:
            return
        counter += len(adj[v])
        dv = d[v]
        for to, w in adj[v]:
            if d[to] is None or d[to] > dv + w:
                d[to] = dv + w
                heapq.heappush(pq, (d[to], to))
    fail(f"counter reaches only {counter}, not past {LIMIT}")


if __name__ == "__main__":
    main()
