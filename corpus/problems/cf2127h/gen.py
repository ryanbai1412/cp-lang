#!/usr/bin/env python3
import sys, random
# seed size: n <= size (capped at 30) per test, sum n^2 <= 900, t <= 50; a random connected simple graph
# grown edge by edge while every vertex stays on at most 5 simple cycles
rng = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 30

def cycle_counts(n, adj, cap):
    # count each simple cycle once, from its smallest vertex, in one direction
    cnt = [0] * (n + 1)
    for s in range(1, n + 1):
        path = [s]; on = [False] * (n + 1); on[s] = True
        def dfs(u):
            for v in adj[u]:
                if v == s and len(path) >= 3 and path[1] < path[-1]:
                    for x in path:
                        cnt[x] += 1
                        if cnt[x] > cap: return True
                elif v > s and not on[v]:
                    on[v] = True; path.append(v)
                    if dfs(v): return True
                    path.pop(); on[v] = False
            return False
        if dfs(s): return None
    return cnt

def graph(n):
    perm = list(range(1, n + 1)); rng.shuffle(perm)
    adj = [set() for _ in range(n + 1)]
    edges = []
    for i in range(1, n):
        p = rng.randint(max(0, i - rng.choice([1, 3, i])), i - 1)
        u, v = perm[p], perm[i]
        adj[u].add(v); adj[v].add(u); edges.append((u, v))
    extra = 4 * n if big else rng.choice([0, 1, 3, n, 4 * n])
    tries = 0
    while extra > 0 and tries < 40 * n:
        tries += 1
        u, v = rng.sample(range(1, n + 1), 2)
        if v in adj[u]: continue
        adj[u].add(v); adj[v].add(u)
        if cycle_counts(n, adj, 5) is None:
            adj[u].discard(v); adj[v].discard(u)
        else:
            edges.append((u, v)); extra -= 1
    rng.shuffle(edges)
    return [(u, v) if rng.random() < .5 else (v, u) for u, v in edges]

if big:
    shape = rng.randint(0, 2)
    ns = [30] if shape == 0 else [4] * 50 if shape == 1 else [21, 21, 3]
else:
    ns = []
    budget = 900
    for _ in range(rng.randint(1, 5)):
        n = rng.randint(3, max(3, min(S, 30)))
        if n * n > budget: break
        ns.append(n); budget -= n * n
print(len(ns))
for n in ns:
    es = graph(n)
    print(n, len(es))
    for u, v in es: print(u, v)
