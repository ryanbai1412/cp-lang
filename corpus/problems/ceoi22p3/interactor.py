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
        self.depth, self.dist = [0] * (n + 1), [0] * (n + 1)
        stack = list(kids[0])
        while stack:
            v = stack.pop()
            p = self.par[v]
            if p:
                self.depth[v], self.dist[v] = self.depth[p] + 1, self.dist[p] + w[v - 1]
            stack += kids[v]

    def lca(self, a, b):
        while self.depth[a] > self.depth[b]:
            a = self.par[a]
        while self.depth[b] > self.depth[a]:
            b = self.par[b]
        while a != b:
            a, b = self.par[a], self.par[b]
        return a

    def to_lca(self, a, b):
        top = self.dist[self.lca(a, b)]
        return self.dist[a] - top, self.dist[b] - top


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
    for a, b in asks:
        io.send(" ".join(map(str, trees[0].to_lca(a, b) + trees[1].to_lca(a, b))))
    pairs = [(chosen[picks[2 * i] - 1], chosen[picks[2 * i + 1] - 1]) for i in range(t)]
    for a, b in pairs:
        io.send(f"{a} {b}")
    for a, b in pairs:
        got = list(map(int, io.recv_line().split()))
        want = [sum(tr.to_lca(a, b)) for tr in trees]
        if got != want:
            raise RuntimeError(f"pair {a} {b}: got {got}, want {want}")


if __name__ == "__main__":
    run_interactor(drive)
