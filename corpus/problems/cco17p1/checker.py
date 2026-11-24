#!/usr/bin/env python3
"""Checker for CCO 2017 Vera and Trail Building: a connected multigraph with 1 <= V, E <= 5000
in which exactly K pairs a < b share a 2-edge-connected component (a round trip using no trail
twice)."""
import sys


def fail(msg):
    sys.stderr.write(msg + "\n")
    sys.exit(1)


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    k = int(open(sys.argv[1]).read().split()[0])
    out = list(map(int, open(sys.argv[2]).read().split()))
    if len(out) < 2:
        fail("no V E line")
    v, e = out[0], out[1]
    if not (1 <= v <= 5000 and 1 <= e <= 5000) or len(out) != 2 + 2 * e:
        fail(f"bad sizes V={v} E={e} with {len(out) - 2} endpoints")
    adj = [[] for _ in range(v + 1)]
    for i in range(e):
        a, b = out[2 + 2 * i], out[3 + 2 * i]
        if not (1 <= a <= v and 1 <= b <= v):
            fail(f"trail {a} {b} out of range")
        adj[a].append((b, i))
        adj[b].append((a, i))
    # bridges by an iterative DFS, skipping only the arc back along the same trail
    tin, low, bridge, timer = [0] * (v + 1), [0] * (v + 1), [False] * e, 1
    tin[1] = low[1] = timer
    stack = [(1, -1, 0)]
    while stack:
        u, via, j = stack.pop()
        if j < len(adj[u]):
            stack.append((u, via, j + 1))
            w, i = adj[u][j]
            if i == via:
                continue
            if tin[w]:
                low[u] = min(low[u], tin[w])
            else:
                timer += 1
                tin[w] = low[w] = timer
                stack.append((w, i, 0))
        elif via >= 0:
            p = next(x for x, i in adj[u] if i == via)
            low[p] = min(low[p], low[u])
            if low[u] > tin[p]:
                bridge[via] = True
    if not all(tin[1:]):
        fail("not connected")
    comp, pairs = [0] * (v + 1), 0
    for s in range(1, v + 1):
        if comp[s]:
            continue
        comp[s], todo, size = s, [s], 0
        while todo:
            u = todo.pop()
            size += 1
            for w, i in adj[u]:
                if not bridge[i] and not comp[w]:
                    comp[w] = s
                    todo.append(w)
        pairs += size * (size - 1) // 2
    if pairs != k:
        fail(f"{pairs} round-trip pairs, wanted {k}")


if __name__ == "__main__":
    main()
