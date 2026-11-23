#!/usr/bin/env python3
"""Checker for CF 2129 A: distinct indices whose pairs reach the reference's
f(S) - g(S). f is the union length; g counts nodes on a cycle, which in a
simple graph are the endpoints of non-bridge edges."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def load():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    return [open(p, encoding="utf-8").read().split() for p in sys.argv[1:]]


def score(pairs):
    cover = set()
    adj = {}
    for k, (a, b) in enumerate(pairs):
        cover.update(range(a, b))
        adj.setdefault(a, []).append((b, k))
        adj.setdefault(b, []).append((a, k))
    tin, low, timer, bridges = {}, {}, 0, set()
    for root in adj:
        if root in tin:
            continue
        stack = [(root, -1, iter(adj[root]))]
        tin[root] = low[root] = timer = timer + 1
        while stack:
            v, pe, it = stack[-1]
            for w, k in it:
                if k == pe:
                    continue
                if w in tin:
                    low[v] = min(low[v], tin[w])
                else:
                    timer += 1
                    tin[w] = low[w] = timer
                    stack.append((w, k, iter(adj[w])))
                    break
            else:
                stack.pop()
                if stack:
                    u = stack[-1][0]
                    low[u] = min(low[u], low[v])
                    if low[v] > tin[u]:
                        bridges.add(pe)
    on_cycle = {x for k, (a, b) in enumerate(pairs) if k not in bridges for x in (a, b)}
    return len(cover) - len(on_cycle)


def main():
    inp, out, ref = load()
    t, pi, po, pr = int(inp[0]), 1, 0, 0
    for case in range(1, t + 1):
        n = int(inp[pi])
        pairs = [(int(inp[pi + 1 + 2 * i]), int(inp[pi + 2 + 2 * i])) for i in range(n)]
        pi += 1 + 2 * n
        k = int(out[po])
        idx = [int(x) for x in out[po + 1:po + 1 + k]]
        po += 1 + k
        rk = int(ref[pr])
        ridx = [int(x) for x in ref[pr + 1:pr + 1 + rk]]
        pr += 1 + rk
        if len(idx) != k or len(set(idx)) != k or any(not 1 <= i <= n for i in idx):
            fail(f"case {case}: indices must be distinct values in 1..{n}")
        got = score([pairs[i - 1] for i in idx])
        best = score([pairs[i - 1] for i in ridx])
        if got != best:
            fail(f"case {case}: f - g = {got}, the optimum is {best}")
    if po != len(out):
        fail(f"{len(out) - po} extra tokens")


if __name__ == "__main__":
    main()
