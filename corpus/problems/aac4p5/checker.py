#!/usr/bin/env python3
"""Checker for AAC4 P5: -1 exactly when the answer is -1; otherwise m distinct directed edges,
no self loops, whose graph has exactly x SCCs with every given pair in one SCC."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def sccs(n, edges):
    adj = [[] for _ in range(n + 1)]
    radj = [[] for _ in range(n + 1)]
    for u, v in edges:
        adj[u].append(v)
        radj[v].append(u)
    order, seen = [], [False] * (n + 1)
    for s in range(1, n + 1):
        if seen[s]:
            continue
        seen[s] = True
        stack = [(s, 0)]
        while stack:
            v, i = stack.pop()
            if i < len(adj[v]):
                stack.append((v, i + 1))
                w = adj[v][i]
                if not seen[w]:
                    seen[w] = True
                    stack.append((w, 0))
            else:
                order.append(v)
    comp, c = [0] * (n + 1), 0
    for s in reversed(order):
        if comp[s]:
            continue
        c += 1
        comp[s] = c
        stack = [s]
        while stack:
            v = stack.pop()
            for w in radj[v]:
                if not comp[w]:
                    comp[w] = c
                    stack.append(w)
    return c, comp


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, m, x, q = inp[:4]
    pairs = [(inp[4 + 2 * i], inp[5 + 2 * i]) for i in range(q)]
    out = list(map(int, open(sys.argv[2]).read().split()))
    ref = open(sys.argv[3]).read().split()
    if ref[:1] == ["-1"] or out[:1] == [-1]:
        if out[:1] != [-1] or ref[:1] != ["-1"] or len(out) != 1:
            fail(f"got {out[:1]}, want {ref[:1]}")
        return
    if len(out) != 2 * m:
        fail(f"expected {m} edges, got {len(out) / 2}")
    edges = [(out[2 * i], out[2 * i + 1]) for i in range(m)]
    if any(not (1 <= u <= n and 1 <= v <= n) or u == v for u, v in edges):
        fail("edge out of range or a self loop")
    if len(set(edges)) != m:
        fail("repeated edge")
    c, comp = sccs(n, edges)
    if c != x:
        fail(f"{c} SCCs, want {x}")
    if any(comp[u] != comp[v] for u, v in pairs):
        fail("a given pair is split")


if __name__ == "__main__":
    main()
