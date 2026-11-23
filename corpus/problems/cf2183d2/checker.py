#!/usr/bin/env python3
"""Checker for CF 2183 D2: the reference's (minimum) number of operations, each
a set of white vertices with distinct depths and no tree edge inside, together
coloring every vertex once."""
from collections import deque
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def main():
    inp, out, ref = load()
    t, pi, po, pr = int(inp[0]), 1, 0, 0
    for case in range(1, t + 1):
        n = int(inp[pi])
        adj = [[] for _ in range(n + 1)]
        for j in range(n - 1):
            u, v = int(inp[pi + 1 + 2 * j]), int(inp[pi + 2 + 2 * j])
            adj[u].append(v)
            adj[v].append(u)
        pi += 2 * n - 1
        depth = [-1] * (n + 1)
        depth[1] = 0
        dq = deque([1])
        while dq:
            u = dq.popleft()
            for v in adj[u]:
                if depth[v] < 0:
                    depth[v] = depth[u] + 1
                    dq.append(v)
        rk = int(ref[pr])
        pr += 1
        for _ in range(rk):
            pr += 1 + int(ref[pr])
        k = int(out[po])
        po += 1
        if k != rk:
            fail(f"case {case}: {k} operations, the minimum is {rk}")
        black = [False] * (n + 1)
        for op in range(1, k + 1):
            m = int(out[po])
            s = [int(x) for x in out[po + 1:po + 1 + m]]
            po += 1 + m
            if len(s) != m or any(not 1 <= x <= n for x in s):
                fail(f"case {case}, op {op}: bad vertex list")
            if len({depth[x] for x in s}) != m:
                fail(f"case {case}, op {op}: two vertices share a depth or repeat")
            ss = set(s)
            for x in s:
                if black[x]:
                    fail(f"case {case}, op {op}: vertex {x} is already black")
                if any(y in ss for y in adj[x]):
                    fail(f"case {case}, op {op}: vertex {x} has a neighbour in the set")
                black[x] = True
        if not all(black[1:]):
            fail(f"case {case}: some vertex stays white")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
