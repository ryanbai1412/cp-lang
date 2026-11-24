#!/usr/bin/env python3
"""Checker for BSSPC '21 S5 James Solves His Tree Problems. The output is -1 exactly when the
reference's is, and otherwise a tree on N vertices, N the reference's (the minimum), that
survives the operations: an L must eat something, a D's edge and endpoints must exist, and what
is left after a D must still be connected."""
import sys
from collections import deque


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def survives(n, edges, ops):
    # The start is a tree; an L keeps it one, and a D keeps it one exactly when it eats one
    # endpoint (or both, and nothing is left), so connectivity needs no search.
    adj = [set() for _ in range(n + 1)]
    for u, v in edges:
        adj[u].add(v)
        adj[v].add(u)
    alive = [True] * (n + 1)
    alive[0] = False
    left = n
    leaves = {v for v in range(1, n + 1) if len(adj[v]) <= 1}

    def eat(v):
        nonlocal left
        alive[v] = False
        left -= 1
        leaves.discard(v)
        for w in adj[v]:
            adj[w].discard(v)
            if len(adj[w]) <= 1:
                leaves.add(w)
        adj[v].clear()

    for i, op in enumerate(ops, 1):
        if op is None:
            if not left:
                return f"operation {i}: L with nothing to eat"
            for v in list(leaves):
                eat(v)
            continue
        a, b = op
        if a > n or b > n or not alive[a] or not alive[b] or b not in adj[a]:
            return f"operation {i}: no toothpick {a}-{b}"
        adj[a].discard(b)
        adj[b].discard(a)
        eaten = 0
        for v in (a, b):
            if len(adj[v]) <= 1:
                leaves.add(v)
            if not adj[v]:
                eat(v)
                eaten += 1
        if eaten == 0 or (eaten == 2 and left):
            return f"operation {i}: the tree falls apart"
    return None


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    tok = open(sys.argv[1], encoding="utf-8").read().split()
    q, ops, p = int(tok[0]), [], 1
    for _ in range(q):
        if tok[p] == "L":
            ops.append(None)
            p += 1
        else:
            ops.append((int(tok[p + 1]), int(tok[p + 2])))
            p += 3
    want = int(open(sys.argv[3], encoding="utf-8").read().split()[0])
    got = open(sys.argv[2], encoding="utf-8").read().split()
    if not got:
        fail("empty output")
    n = int(got[0])
    if want == -1 or n == -1:
        if n != want:
            fail(f"printed {n}, the answer is {want}")
        return
    if n != want:
        fail(f"N = {n}, the minimum is {want}")
    if len(got) != 2 * n - 1:
        fail(f"expected {n - 1} edges")
    edges = [(int(got[i]), int(got[i + 1])) for i in range(1, len(got), 2)]
    parent = list(range(n + 1))

    def find(v):
        while parent[v] != v:
            parent[v] = parent[parent[v]]
            v = parent[v]
        return v

    for u, v in edges:
        if not (1 <= u <= n and 1 <= v <= n) or find(u) == find(v):
            fail(f"edge {u} {v} is out of range or closes a cycle")
        parent[find(u)] = find(v)
    why = survives(n, edges, ops)
    if why:
        fail(why)


if __name__ == "__main__":
    main()
