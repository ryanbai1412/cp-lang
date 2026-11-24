#!/usr/bin/env python3
"""Checker for IOI 2019 Split the Attractions: all zeros exactly when the answer is all
zeros; otherwise a, b and n - a - b vertices labelled 1, 2, 3, at least two sets connected."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def connected(n, adj, part, label):
    members = [v for v in range(n) if part[v] == label]
    if not members:
        return False
    seen = {members[0]}
    stack = [members[0]]
    while stack:
        v = stack.pop()
        for w in adj[v]:
            if part[w] == label and w not in seen:
                seen.add(w)
                stack.append(w)
    return len(seen) == len(members)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp = list(map(int, open(sys.argv[1]).read().split()))
    n, m, a, b = inp[:4]
    adj = [[] for _ in range(n)]
    for i in range(m):
        u, v = inp[4 + 2 * i], inp[5 + 2 * i]
        adj[u].append(v)
        adj[v].append(u)
    out = list(map(int, open(sys.argv[2]).read().split()))
    ref = list(map(int, open(sys.argv[3]).read().split()))
    if len(out) != n:
        fail(f"expected {n} labels, got {len(out)}")
    if all(x == 0 for x in ref) or all(x == 0 for x in out):
        if out != ref:
            fail("disagrees with the reference on whether a split exists")
        return
    want = {1: a, 2: b, 3: n - a - b}
    for label, size in want.items():
        if out.count(label) != size:
            fail(f"set {label} has {out.count(label)} vertices, want {size}")
    if sum(connected(n, adj, out, label) for label in want) < 2:
        fail("fewer than two sets are connected")


main()
