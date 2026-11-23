#!/usr/bin/env python3
"""Checker for CF 2187 G: a permutation q whose arrays p_1..p_n (each raising
the minimum by n) have Cartesian-tree child-parent edges whose union is s."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def edges(a, acc):
    stack = []
    parent = [-1] * len(a)
    for i, v in enumerate(a):
        last = -1
        while stack and a[stack[-1]] < v:
            last = stack.pop()
        if last >= 0:
            parent[last] = i
        if stack:
            parent[i] = stack[-1]
        stack.append(i)
    for i, p in enumerate(parent):
        if p >= 0:
            acc.add((i, p))


def main():
    inp, out, _ = load()
    t, pi, po = int(inp[0]), 1, 0
    for case in range(1, t + 1):
        n = int(inp[pi])
        rows = inp[pi + 1:pi + 1 + n]
        pi += 1 + n
        q = [int(x) for x in out[po:po + n]]
        po += n
        if len(q) != n or sorted(q) != list(range(1, n + 1)):
            fail(f"case {case}: not a permutation of 1..{n}")
        want = {(i, j) for i in range(n) for j in range(n) if rows[i][j] == "1"}
        got, p = set(), q[:]
        order = sorted(range(n), key=lambda i: q[i])
        for step in range(n):
            edges(p, got)
            p[order[step]] += n
        if got != want:
            diff = sorted(got ^ want)[0]
            fail(f"case {case}: edge {diff[0] + 1} -> {diff[1] + 1} differs from s")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
