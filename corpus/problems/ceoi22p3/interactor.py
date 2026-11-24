#!/usr/bin/env python3
"""Judge for CEOI 2022 P3 (Prize): sends N K Q T and both trees' parents, reads K distinct
chosen labels, at most Q questions "? a b" on chosen labels and "!", answers each with the
distances of a and b to their LCA in tree 1 then tree 2, sends the T pairs of chosen labels
and checks the two distances printed for each."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "lib"))
from interactor_lib import run_interactor


class Tree:
    def __init__(self, par, w):
        n = len(par)
        self.par = [0] + [0 if p == -1 else p for p in par]
        kids = [[] for _ in range(n + 1)]
        for v in range(1, n + 1):
            kids[self.par[v]].append(v)
        self.dist = [0] * (n + 1)
        self.kids = kids
        stack = list(kids[0])
        while stack:
            v = stack.pop()
            p = self.par[v]
            if p:
                self.dist[v] = self.dist[p] + w[v - 1]
            stack += kids[v]

    def lcas(self, pairs):
        """Offline Tarjan: the LCA of every pair, in one walk of the tree."""
        n = len(self.par) - 1
        asked = [[] for _ in range(n + 1)]
        for i, (a, b) in enumerate(pairs):
            asked[a].append((b, i))
            asked[b].append((a, i))
        up, done, got = list(range(n + 1)), bytearray(n + 1), [0] * len(pairs)

        def find(x):
            r = x
            while up[r] != r:
                r = up[r]
            while up[x] != r:
                up[x], x = r, up[x]
            return r

        nxt, stack = [0] * (n + 1), [0]
        while stack:
            v = stack[-1]
            kids = self.kids[v]
            if nxt[v] < len(kids):
                stack.append(kids[nxt[v]])
                nxt[v] += 1
                continue
            stack.pop()
            done[v] = 1
            for u, i in asked[v]:
                if done[u]:
                    got[i] = find(u)
            if stack:
                up[v] = stack[-1]
        return got

    def to_lca(self, pairs):
        tops = self.lcas(pairs)
        return [(self.dist[a] - self.dist[c], self.dist[b] - self.dist[c]) for (a, b), c in zip(pairs, tops)]


def drive(case_text, io):
    tok = list(map(int, case_text.split()))
    n, k, q, t = tok[:4]
    at = 4
    trees, parents = [], []
    for _ in range(2):
        par, w = tok[at:at + n], tok[at + n:at + 2 * n]
        at += 2 * n
        trees.append(Tree(par, w))
        parents.append(par)
    picks = tok[at:at + 2 * t]
    io.send(f"{n} {k} {q} {t}")
    for par in parents:
        io.send(" ".join(map(str, par)))
    chosen = list(map(int, io.recv_line().split()))
    if len(chosen) != k or len(set(chosen)) != k or not all(1 <= v <= n for v in chosen):
        raise RuntimeError(f"want {k} distinct labels in 1..={n}")
    picked = set(chosen)
    asks = []
    while True:
        parts = io.recv_line().split()
        if parts == ["!"]:
            break
        if len(parts) != 3 or parts[0] != "?":
            raise RuntimeError(f"unexpected {parts[:4]}")
        a, b = int(parts[1]), int(parts[2])
        if a not in picked or b not in picked:
            raise RuntimeError(f"? {a} {b} asks an unchosen label")
        asks.append((a, b))
        if len(asks) > q:
            raise RuntimeError(f"more than {q} questions")
    pairs = [(chosen[picks[2 * i] - 1], chosen[picks[2 * i + 1] - 1]) for i in range(t)]
    first, second = (tr.to_lca(asks + pairs) for tr in trees)
    for one, two in zip(first, second[:len(asks)]):
        io.send(" ".join(map(str, one + two)))
    for a, b in pairs:
        io.send(f"{a} {b}")
    for (a, b), one, two in zip(pairs, first[len(asks):], second[len(asks):]):
        got = list(map(int, io.recv_line().split()))
        want = [sum(one), sum(two)]
        if got != want:
            raise RuntimeError(f"pair {a} {b}: got {got}, want {want}")


if __name__ == "__main__":
    run_interactor(drive)
