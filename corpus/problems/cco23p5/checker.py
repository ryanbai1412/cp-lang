#!/usr/bin/env python3
# checker.py input output answer: the answer's profit, then a route of distinct cities from city 1,
# each next one at most K roads from the last, whose profits sum to it.
import sys
from collections import deque
tok = open(sys.argv[1]).read().split()
n, k = int(tok[0]), int(tok[1])
adj = [[] for _ in range(n + 1)]
for i in range(n - 1):
    u, v = int(tok[2 + 2 * i]), int(tok[3 + 2 * i]); adj[u].append(v); adj[v].append(u)
p = [0] + list(map(int, tok[2 * n:3 * n]))
par = [0] * (n + 1); dep = [0] * (n + 1); seen = [False] * (n + 1); seen[1] = True; q = deque([1])
while q:
    u = q.popleft()
    for v in adj[u]:
        if not seen[v]: seen[v] = True; par[v] = u; dep[v] = dep[u] + 1; q.append(v)
out = list(map(int, open(sys.argv[2]).read().split()))
best = int(open(sys.argv[3]).read().split()[0])
if out[0] != best: sys.exit(f"profit {out[0]}, best {best}")
m = out[1]; route = out[2:]
if len(route) != m or m < 1: sys.exit(f"{len(route)} cities for M = {m}")
if route[0] != 1: sys.exit("the route starts at city 1")
if len(set(route)) != m or not all(1 <= c <= n for c in route): sys.exit("a city repeats or is out of range")
for a, b in zip(route, route[1:]):
    steps = 0
    while a != b and steps <= k:
        if dep[a] < dep[b]: a, b = b, a
        a = par[a]; steps += 1
    if a != b or steps > k: sys.exit(f"cities more than {k} roads apart")
if sum(p[c] for c in route) != best: sys.exit("the route's profits do not sum to the printed profit")
print("ok")
