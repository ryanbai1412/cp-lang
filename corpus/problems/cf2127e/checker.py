#!/usr/bin/env python3
"""Checker for CF 2127 E: the reference's minimum cost, and a coloring that
keeps every given color, uses 1..k, and has that cost. A vertex is cutie when
two of its child subtrees share a color other than its own; child color sets
merge small-to-large."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def cost(n, w, c, children):
    order, stack = [], [1]
    while stack:
        v = stack.pop()
        order.append(v)
        stack.extend(children[v])
    sets, total = [None] * (n + 1), 0
    for v in reversed(order):
        big, shared = set(), set()
        for u in children[v]:
            s = sets[u]
            sets[u] = None
            if len(s) > len(big):
                big, s = s, big
            for col in s:
                if col in big:
                    shared.add(col)
                else:
                    big.add(col)
        if shared - {c[v]}:
            total += w[v]
        big.add(c[v])
        sets[v] = big
    return total


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    inp, out, ref = (open(p, encoding="utf-8").read().split() for p in sys.argv[1:])
    t, pi, po, pr = int(inp[0]), 1, 0, 0
    for case in range(1, t + 1):
        n, k = int(inp[pi]), int(inp[pi + 1])
        w = [0] + [int(x) for x in inp[pi + 2:pi + 2 + n]]
        c0 = [0] + [int(x) for x in inp[pi + 2 + n:pi + 2 + 2 * n]]
        adj = [[] for _ in range(n + 1)]
        base = pi + 2 + 2 * n
        for j in range(n - 1):
            u, v = int(inp[base + 2 * j]), int(inp[base + 2 * j + 1])
            adj[u].append(v)
            adj[v].append(u)
        pi = base + 2 * (n - 1)
        children, seen, stack = [[] for _ in range(n + 1)], [False] * (n + 1), [1]
        seen[1] = True
        while stack:
            v = stack.pop()
            for u in adj[v]:
                if not seen[u]:
                    seen[u] = True
                    children[v].append(u)
                    stack.append(u)
        want, got = int(ref[pr]), int(out[po])
        c = [0] + [int(x) for x in out[po + 1:po + 1 + n]]
        po, pr = po + 1 + n, pr + 1 + n
        if got != want:
            fail(f"case {case}: printed cost {got}, the minimum is {want}")
        if len(c) != n + 1 or any(not 1 <= c[i] <= k or (c0[i] and c[i] != c0[i]) for i in range(1, n + 1)):
            fail(f"case {case}: the coloring changes a given color or leaves 1..{k}")
        real = cost(n, w, c, children)
        if real != got:
            fail(f"case {case}: the coloring costs {real}, not {got}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
