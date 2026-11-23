#!/usr/bin/env python3
"""Checker for CF 2147 H: the reference's (minimum) number of colors, a
partition of 1..n, and each induced subgraph good: some d >= 2 divides every
pairwise max flow (d divides 0, so all-zero flows are good). Those values are the edge weights of a Gomory-Hu tree
(Gusfield's construction), so their gcd decides it."""
import sys
from collections import deque
from math import gcd


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def max_flow(n, cap, s, t):
    flow = [row[:] for row in cap]
    total = 0
    while True:
        par = [-1] * n
        par[s] = s
        dq = deque([s])
        while dq and par[t] < 0:
            u = dq.popleft()
            for v in range(n):
                if par[v] < 0 and flow[u][v] > 0:
                    par[v] = u
                    dq.append(v)
        if par[t] < 0:
            return total, [par[v] >= 0 for v in range(n)]
        f, v = float("inf"), t
        while v != s:
            f = min(f, flow[par[v]][v])
            v = par[v]
        v = t
        while v != s:
            flow[par[v]][v] -= f
            flow[v][par[v]] += f
            v = par[v]
        total += f


def good(cap):
    n = len(cap)
    parent, g = [0] * n, 0
    for s in range(1, n):
        f, side = max_flow(n, cap, s, parent[s])
        g = gcd(g, f)
        for v in range(s + 1, n):
            if side[v] and parent[v] == parent[s]:
                parent[v] = s
    return g != 1


def main():
    inp, out, ref = load()
    t, pi, po = int(inp[0]), 1, 0
    for case in range(1, t + 1):
        n, m = int(inp[pi]), int(inp[pi + 1])
        w = {}
        for j in range(m):
            u, v, c = (int(x) for x in inp[pi + 2 + 3 * j:pi + 5 + 3 * j])
            w[(u, v)] = w[(v, u)] = c
        pi += 2 + 3 * m
        c = int(out[po])
        po += 1
        seen = []
        for _ in range(c):
            k = int(out[po])
            part = [int(x) for x in out[po + 1:po + 1 + k]]
            po += 1 + k
            seen += part
            cap = [[w.get((a, b), 0) for b in part] for a in part]
            if not good(cap):
                fail(f"case {case}: color class {part[:10]} is not good")
        if sorted(seen) != list(range(1, n + 1)):
            fail(f"case {case}: colors do not partition 1..{n}")
    pr = po2 = 0
    for case in range(1, t + 1):
        rc, c = int(ref[pr]), int(out[po2])
        pr += 1
        po2 += 1
        for _ in range(rc):
            pr += 1 + int(ref[pr])
        for _ in range(c):
            po2 += 1 + int(out[po2])
        if c != rc:
            fail(f"case {case}: {c} colors, the minimum is {rc}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
