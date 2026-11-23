#!/usr/bin/env python3
# checker.py input output [answer]: the reference's number of new links (the minimum,
# ceil(L / 2) for L leaves), each between two different computers, and no tree edge left a
# bridge once they are added.
import sys
t = list(map(int, open(sys.argv[1]).read().split()))
n = t[0]; es = [(t[1 + 2 * i], t[2 + 2 * i]) for i in range(n - 1)]
out = list(map(int, open(sys.argv[2]).read().split()))
k = out[0]; links = [(out[1 + 2 * i], out[2 + 2 * i]) for i in range(k)]
if len(out) != 1 + 2 * k: sys.exit("wrong token count")
if len(sys.argv) > 3:
    want = int(open(sys.argv[3]).read().split()[0])
    if k != want: sys.exit(f"{k} links, minimum {want}")
adj = [[] for _ in range(n + 1)]
for u, v in es: adj[u].append(v); adj[v].append(u)
par = [0] * (n + 1); order = [1]; seen = [False] * (n + 1); seen[1] = True
for u in order:
    for v in adj[u]:
        if not seen[v]: seen[v] = True; par[v] = u; order.append(v)
# a tree edge (v, par v) is covered iff some link has exactly one end in v's subtree:
# cover[v] = number of link ends in the subtree, minus twice the links whose lca is inside
tin = [0] * (n + 1); depth = [0] * (n + 1)
for u in order[1:]: depth[u] = depth[par[u]] + 1
up = [par[:]]
for _ in range(20): up.append([up[-1][up[-1][v]] for v in range(n + 1)])
def lca(a, b):
    if depth[a] < depth[b]: a, b = b, a
    d = depth[a] - depth[b]
    for j in range(21):
        if d >> j & 1: a = up[j][a]
    if a == b: return a
    for j in range(20, -1, -1):
        if up[j][a] != up[j][b]: a, b = up[j][a], up[j][b]
    return par[a]
cnt = [0] * (n + 1)
for a, b in links:
    if not (1 <= a <= n and 1 <= b <= n) or a == b: sys.exit(f"bad link {a} {b}")
    cnt[a] += 1; cnt[b] += 1; cnt[lca(a, b)] -= 2
for u in reversed(order[1:]):
    if cnt[u] == 0: sys.exit(f"edge {u} {par[u]} is still a bridge")
    cnt[par[u]] += cnt[u]
print("ok")
